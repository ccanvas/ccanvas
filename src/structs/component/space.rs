use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::traits::Component;

use crate::structs::*;
use crate::values::FOCUSED;

/// the basic unit of display
pub struct Space {
    /// name of the current space
    label: String,

    /// unique identifier of the current space - a "path" of u32s
    discrim: Discriminator,

    /// data storage for children
    pool: Arc<Mutex<Pool>>,

    /// shared storage folder for space children
    storage: Storage,

    /// spaces the current space contains
    subspaces: Arc<Mutex<Collection<Self>>>,

    /// currently in use space, could be self or children
    focus: Arc<Mutex<Focus>>,

    /// process event subscriptions in this space
    passes: Arc<Mutex<Passes>>,

    /// process pool
    processes: Arc<Mutex<Collection<Process>>>,
    // processes: Arc<Mutex<Collection<Process>>>,
}

impl Space {
    pub async fn new(label: String) -> Self {
        Self::new_with_parent(label, &Discriminator::default()).await
    }

    /// create new self with parent discriminator
    async fn new_with_parent(label: String, parent_discrim: &Discriminator) -> Self {
        Self {
            storage: Storage::new(parent_discrim).await,
            label,
            discrim: parent_discrim.new_child(),
            pool: Arc::new(Mutex::new(Pool::default())),
            subspaces: Arc::new(Mutex::new(Collection::default())),
            focus: Arc::new(Mutex::new(Focus::default())),
            passes: Arc::new(Mutex::new(Passes::default())),
            processes: Arc::new(Mutex::new(Collection::default())),
        }
    }

    /// start listening to all events, only the top level,
    /// "master" space should do this
    pub async fn listen(arc: Arc<Self>) {
        let mut listener = Event::start();

        while let Some(mut event) = listener.recv().await {
            // drop for quitting the entire application
            if let Event::RequestPacket(req) = &mut event {
                if req.get().content()
                    == &(RequestContent::Drop {
                        discrim: Some(Discriminator(vec![1])),
                    })
                    && req.get().target().0.is_empty()
                {
                    return;
                }
            }

            let arc = arc.clone();
            // pass the event to master space
            tokio::spawn(async move {
                arc.pass(&mut event).await;
            });
        }
    }

    /// insert a new process
    pub async fn spawn(
        &self,
        label: String,
        command: String,
        args: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        self.processes
            .lock()
            .await
            .insert(Process::spawn(label, &self.discrim, command, args).await?);
        Ok(())
    }
}

#[async_trait]
impl Component for Space {
    fn label(&self) -> &str {
        &self.label
    }

    fn discrim(&self) -> &Discriminator {
        &self.discrim
    }

    fn storage(&self) -> &Storage {
        &self.storage
    }

    async fn pass(&self, event: &mut Event) -> Unevaluated<bool> {
        #[cfg(feature = "log")]
        log::debug!("{:?} got event {event:?}", self.discrim);
        match event {
            // if the target is self
            Event::RequestPacket(req) if req.get().target() == self.discrim() => {
                match req.get().content() {
                    RequestContent::RemoveEntry { label } => {
                        self.pool.lock().await.remove(label, &self.discrim);
                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::RemovedValue,
                            },
                            *req.get().id(),
                        ));
                    }
                    RequestContent::Unwatch { label, watcher } => {
                        // remove watche from pool entry
                        let res =
                            if self
                                .pool
                                .lock()
                                .await
                                .unwatch(label, watcher, self.discrim.clone())
                            {
                                ResponseContent::Success {
                                    content: ResponseSuccess::Unwatched,
                                }
                            } else {
                                ResponseContent::Error {
                                    content: ResponseError::EntryNotFound,
                                }
                            };
                        let _ = req.respond(Response::new_with_request(res, *req.get().id()));
                    }
                    RequestContent::SetEntry { label, value } => {
                        // set value of a pool entry
                        // this never fails
                        self.pool.lock().await.set(label, value.clone());
                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::ValueSet,
                            },
                            *req.get().id(),
                        ));
                    }
                    RequestContent::WatchInternal(watch) => {
                        // add a watch item
                        self.pool.lock().await.watch(
                            watch.label.clone(),
                            watch.sender.clone(),
                            watch.watcher.clone(),
                        );
                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::Watching,
                            },
                            *req.get().id(),
                        ));
                    }
                    RequestContent::GetEntry { label } => {
                        let res = match self.pool.lock().await.get(label) {
                            Some(value) => ResponseContent::Success {
                                content: ResponseSuccess::Value { value },
                            },
                            None => ResponseContent::Error {
                                content: ResponseError::EntryNotFound,
                            },
                        };
                        let _ = req.respond(Response::new_with_request(res, *req.get().id()));
                    }
                    RequestContent::FocusAt => {
                        // switch focus to that space, and only that space
                        #[cfg(feature = "log")]
                        log::debug!("{:?} locking focus", self.discrim);
                        let mut focus = self.focus.lock().await;
                        #[cfg(feature = "log")]
                        log::debug!("{:?} locked focus", self.discrim);
                        if let Focus::Children(discrim) = &*focus {
                            #[cfg(feature = "log")]
                            log::debug!("{:?} locking subspaces", self.discrim);
                            self.subspaces
                                .lock()
                                .await
                                .find_by_discrim(discrim)
                                .unwrap()
                                .pass(&mut Event::Unfocus)
                                .await;
                            #[cfg(feature = "log")]
                            log::debug!("{:?} locked subspaces", self.discrim);
                            #[cfg(feature = "log")]
                            log::debug!("{:?} unlocked subspaces", self.discrim);
                            *focus = Focus::This;
                        }
                        #[cfg(feature = "log")]
                        log::debug!("{:?} unlocked focus", self.discrim);

                        *FOCUSED.get().unwrap().lock().unwrap() = self.discrim.clone();
                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::FocusChanged,
                            },
                            *req.get().id(),
                        ));

                        return false.into();
                    }
                    RequestContent::NewSpace { label } => {
                        // just add a new entry to subspaces
                        let space = Space::new_with_parent(label.clone(), &self.discrim).await;
                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::SpaceCreated {
                                    discrim: space.discrim.clone(),
                                },
                            },
                            *req.get().id(),
                        ));
                        self.subspaces.lock().await.insert(space);
                    }
                    // spawn a new process
                    RequestContent::Spawn {
                        command,
                        args,
                        label,
                    } => {
                        // check if spawning process succeed
                        match Process::spawn(
                            label.clone(),
                            &self.discrim,
                            command.clone(),
                            args.clone(),
                        )
                        .await
                        {
                            Ok(process) => {
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Success {
                                        content: ResponseSuccess::Spawned {
                                            discrim: process.discrim().clone(),
                                        },
                                    },
                                    *req.get().id(),
                                ));
                                self.processes.lock().await.insert(process);
                            }
                            Err(_) => {
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Error {
                                        content: ResponseError::SpawnFailed,
                                    },
                                    *req.get().id(),
                                ));
                            }
                        }
                    }
                    // add an item to passes
                    RequestContent::Subscribe {
                        channel,
                        priority,
                        component: Some(discrim),
                    } => {
                        // checks if the discrim is to a valid process
                        if let Some(child) = self.discrim.immediate_child(discrim.clone()) {
                            if self.processes.lock().await.contains(&child) {
                                // if its a process, subscribe to the event right here
                                self.passes.lock().await.subscribe(
                                    channel.clone(),
                                    PassItem::new(discrim.clone(), *priority),
                                );
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Success {
                                        content: ResponseSuccess::SubscribeAdded,
                                    },
                                    *req.get().id(),
                                ));
                            } else {
                                // or else just throw a not found
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Error {
                                        content: ResponseError::ComponentNotFound,
                                    },
                                    *req.get().id(),
                                ));
                            }
                        }
                    }
                    // remove an item from
                    RequestContent::Unsubscribe {
                        channel,
                        component: Some(discrim),
                    } => {
                        // checks if the discrim is to a valid process
                        if let Some(child) = self.discrim.immediate_child(discrim.clone()) {
                            if self.processes.lock().await.contains(&child) {
                                // if its a process, subscribe to the event right here
                                self.passes
                                    .lock()
                                    .await
                                    .unsubscribe(channel.clone(), discrim);
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Success {
                                        content: ResponseSuccess::SubscribeRemoved,
                                    },
                                    *req.get().id(),
                                ));
                            } else {
                                // or else just throw a not found
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Error {
                                        content: ResponseError::ComponentNotFound,
                                    },
                                    *req.get().id(),
                                ));
                            }
                        }
                    }
                    RequestContent::Drop { discrim } => {
                        // drop (remove) a child component
                        if let Some(child) = self.discrim.immediate_child(discrim.clone().unwrap())
                        {
                            if self.processes.lock().await.remove(&child) {
                                // if its a process, then remove all of its passes
                                self.passes.lock().await.unsub_all(&child);
                            } else if self.subspaces.lock().await.remove(&child) {
                                if *self.focus.lock().await == Focus::Children(child) {
                                    // if the removed space is currently focused, then switch focus
                                    // to parent space
                                    *self.focus.lock().await = Focus::This
                                }
                            } else {
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Error {
                                        content: ResponseError::ComponentNotFound,
                                    },
                                    *req.get().id(),
                                ));
                                return false.into();
                            }
                            let _ = req.respond(Response::new_with_request(
                                ResponseContent::Success {
                                    content: ResponseSuccess::Dropped,
                                },
                                *req.get().id(),
                            ));
                        }
                    }
                    RequestContent::Render { content, flush } => {
                        // does rendering stuff, no explainations needed
                        let flush = *flush;
                        let content = content.clone();
                        tokio::task::spawn_blocking(move || content.draw(flush))
                            .await
                            .unwrap();

                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::Rendered,
                            },
                            *req.get().id(),
                        ));
                    }
                    RequestContent::Message {
                        content,
                        sender,
                        target,
                    } => {
                        // heres all the things needed to construct an event
                        let sender = sender.clone();
                        let target = target.clone();
                        let content = content.clone();

                        let _ = req.respond(Response::new_with_request(
                            ResponseContent::Success {
                                content: ResponseSuccess::MessageDelivered,
                            },
                            *req.get().id(),
                        ));

                        // now pass the event to self
                        *event = Event::Message {
                            sender,
                            target,
                            content,
                        };

                        self.pass(event).await;
                    }
                    RequestContent::GetState { .. }
                    | RequestContent::Watch { .. }
                    | RequestContent::Subscribe {
                        component: None, ..
                    }
                    | RequestContent::Unsubscribe {
                        component: None, ..
                    } => unreachable!("impossible requests"),
                    RequestContent::ConfirmRecieve { .. } | RequestContent::SetSocket { .. } => {
                        unreachable!("not requests to spaces")
                    }
                }

                return false.into();
            } // do stuff
            Event::RequestPacket(req) => {
                // pass the event to "next immediate child"
                // aka the next item it should pass to in order to get the request
                // to its intended target
                if let Some(child) = self.discrim.immediate_child(req.get().target().clone()) {
                    if req.get().content() == &RequestContent::FocusAt {
                        let mut focus = self.focus.lock().await;
                        let subspaces = self.subspaces.lock().await;

                        if !subspaces.contains(&child) {
                            let _ = req.respond(Response::new_with_request(
                                ResponseContent::Error {
                                    content: ResponseError::ComponentNotFound,
                                },
                                *req.get().id(),
                            ));
                        }

                        if let Focus::Children(focused) = &*focus {
                            if req.get().target().starts_with(focused) {
                                let subspace = subspaces.find_by_discrim_arc(&child).unwrap();
                                subspace.pass(event).await;
                            } else {
                                subspaces
                                    .find_by_discrim(focused)
                                    .unwrap()
                                    .pass(&mut Event::Unfocus)
                                    .await;
                                *focus = Focus::Children(child.clone());

                                let child = subspaces.find_by_discrim(&child).unwrap();
                                // this kinda defeats the point of non blocking
                                // as this might block
                                // but it shouldnt, and let's just pray on it that its true
                                //
                                // event must come after the actual focus or else it might get
                                // passed to the wrong components
                                child.pass(event).await.evaluate().await;
                                child.pass(&mut Event::Focus).await;
                            }
                        } else {
                            *focus = Focus::Children(child.clone());
                            let child = subspaces.find_by_discrim(&child).unwrap();
                            child.pass(event).await.evaluate().await;
                            child.pass(&mut Event::Focus).await;
                        }

                        return false.into();
                    }

                    // no 2 components are the same, so order shouldnt matter
                    if let Some(proc) = self.processes.lock().await.find_by_discrim(&child) {
                        if let Some(subscriptions) = req.get().subscriptions() {
                            if self
                                .passes
                                .lock()
                                .await
                                .subscribers(subscriptions)
                                .iter()
                                .any(|item| item.discrim() == proc.discrim())
                            {
                                proc.pass(event).await;
                            } else {
                                let _ = req.respond(Response::new_with_request(
                                    ResponseContent::Undelivered,
                                    *req.get().id(),
                                ));
                            }
                        } else {
                            proc.pass(event).await;
                        }
                    } else if let Some(space) =
                        self.subspaces.lock().await.find_by_discrim_arc(&child)
                    {
                        space.pass(event).await;
                    } else {
                        let _ = req.respond(Response::new(ResponseContent::Undelivered));
                    }

                    return false.into();
                }
                // otherwise self is not a parent to the target component
                // and something went wrong
            }
            _ => {}
        }

        // all components listening to this event
        let targets = self.passes.lock().await.subscribers(&event.subscriptions());

        let processes = self.processes.clone();
        let mut event = event.clone();
        let subspaces = self.subspaces.clone();
        let focus = self.focus.clone();
        #[cfg(feature = "log")]
        let discrim = self.discrim.clone();
        let uneval = tokio::spawn(async move {
            // repeat until someone decide to capture the event
            for target in targets {
                #[cfg(feature = "log")]
                log::debug!("passing {event:?} to {target:?}");
                let res = processes
                    .lock()
                    .await
                    .find_by_discrim_arc(target.discrim())
                    .unwrap()
                    .pass(&mut event)
                    .await;
                let res = res.evaluate().await;
                if !res {
                    #[cfg(feature = "log")]
                    log::debug!("event {event:?} captured by {target:?}");
                    return false;
                }
            }

            if event == Event::Focus {
                return false;
            }

            // if all went well then continue to pass down into subspaces
            let focus = focus.lock().await.clone();
            if let Focus::Children(discrim) = focus {
                let subspace = subspaces
                    .lock()
                    .await
                    .find_by_discrim_arc(&discrim)
                    .unwrap();
                subspace.pass(&mut event).await.evaluate().await;
            }

            true
        });

        Unevaluated::Unevaluated(uneval)
    }
}
