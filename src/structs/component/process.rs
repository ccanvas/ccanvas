use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    env,
    ffi::OsString,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::Stdio,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::{
    process::{Child, Command},
    sync::{
        mpsc::{self, UnboundedSender},
        oneshot, Mutex, OnceCell,
    },
    task::JoinHandle,
};

use crate::{traits::Component, values::FOCUSED};

use crate::structs::*;

/// single runnable process
pub struct Process {
    /// name of the current process
    label: String,

    /// unique identifier of the current process
    discrim: Discriminator,

    /// data storage for self
    pool: Arc<Mutex<Pool>>,

    /// shared storage folder for self
    storage: Storage,

    /// command that was ran
    command: Vec<String>,

    /// process handle
    child: Arc<Mutex<Child>>,

    /// handle to the task responsible for listening to requests
    listener: JoinHandle<()>,

    /// handle to the task responsible for responding
    responder: JoinHandle<()>,

    /// path to response socket
    res: UnboundedSender<Response>,

    /// event confirm recieve senders
    confirm_handles: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>>,

    /// channel suppressors
    suppressors: Arc<Mutex<Suppressors>>,
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.discrim == other.discrim
    }
}

static ENVS: OnceCell<Vec<(OsString, OsString)>> = OnceCell::const_new();

impl Process {
    /// spawns a new process with command
    pub async fn spawn(
        label: String,
        parent: &Discriminator,
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    ) -> Result<Self, std::io::Error> {
        let discrim = parent.new_child();
        let storage = Storage::new(&discrim).await;

        // a new sender is pushed to the map whenever something is sent to the component
        // the sender returns a boolean, if true then the event will not be captured, vice versa
        // the important part is that this will hold the process until a confirmation message is
        // recieved
        let confirm_handles: Arc<Mutex<HashMap<u32, oneshot::Sender<bool>>>> =
            Arc::new(Mutex::new(HashMap::default()));

        // the component should send requests to this path
        let socket_path = storage.path().join("requests.sock");
        Storage::remove_if_exist(&socket_path).await.unwrap();
        let child = Arc::new(Mutex::new(
            Command::new(&command)
                .envs(
                    ENVS.get_or_init(|| async {
                        tokio::task::spawn_blocking(|| env::vars_os().collect::<Vec<_>>())
                            .await
                            .unwrap()
                    })
                    .await
                    .clone()
                    .into_iter(),
                )
                .envs(env.into_iter())
                .envs(std::env::vars_os())
                .env("CCANVAS_COMPONENT", "1")
                .kill_on_drop(true)
                .args(&args)
                .current_dir(storage.path())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?,
        ));

        let (responder_send, mut responder_recv): (UnboundedSender<Response>, _) =
            mpsc::unbounded_channel();

        let (set_socket_send, set_socket_recv): (oneshot::Sender<()>, _) = oneshot::channel();

        // the responder task recieve Response
        // serialise it and send it to the component, if it specified a socket to send to
        let responder = {
            let confirm_handles = confirm_handles.clone();
            let child = child.clone();
            let discrim = discrim.clone();
            tokio::spawn(async move {
                // by default there is no socket
                let mut socket = None;
                let mut socket_confirm = Some(set_socket_send);

                while let Some(res) = responder_recv.recv().await {
                    let confirm_handles = confirm_handles.clone();
                    // this special "response" is recieved when a SetSocket request
                    // is sent by the component
                    if let ResponseContent::SetSocket(path) = res.content() {
                        socket = Some(path.to_owned());
                        if socket_confirm.is_some() {
                            let _ = std::mem::take(&mut socket_confirm).unwrap().send(());
                        }
                        continue;
                    }

                    // only send a message when a socket is specified
                    if let Some(socket) = &socket {
                        #[cfg(feature = "log")]
                        log::info!("{discrim:?} sent {res:?}");
                        let socket = socket.clone();
                        let child = child.clone();
                        let discrim = discrim.clone();
                        tokio::spawn(async move {
                            // check if the child process has crashed
                            if child.lock().await.try_wait().unwrap().is_some() {
                                Event::send(Event::RequestPacket(
                                    Packet::new(Request::new(
                                        discrim.clone().immediate_parent().unwrap(),
                                        RequestContent::Drop {
                                            // if true, then tell the parent space to drop it
                                            discrim: Some(discrim),
                                        },
                                    ))
                                    .0,
                                ))
                            }

                            #[cfg(feature = "log")]
                            log::debug!("sent {res:?}");

                            if let Ok(mut stream) = UnixStream::connect(socket) {
                                stream
                                    .write_all(serde_json::to_vec(&res).unwrap().as_slice())
                                    .unwrap();
                                stream.flush().unwrap();
                            } else {
                                // if send failed, it is impossible to get a response message
                                // so manually remove it to trigger an undelivered response
                                confirm_handles.lock().await.remove(&res.id());
                            }
                        });
                    }
                }
            })
        };

        // the listener listens to event and handles it, that all
        let listener = {
            let discrim = discrim.clone();
            let confirm_handles = confirm_handles.clone();
            let responder = responder_send.clone();
            let storage = storage.clone();
            tokio::spawn(async move {
                // creates a socket and listens to it
                let socket =
                    tokio::task::block_in_place(|| UnixListener::bind(socket_path).unwrap());
                let mut incoming = socket.incoming();

                while let Some(stream) = tokio::task::block_in_place(|| incoming.next()) {
                    // give up if the stream is errorneous
                    let mut stream = match stream {
                        Ok(stream) => stream,
                        Err(_) => continue,
                    };

                    let mut msg = String::new();
                    // another chance to give up
                    if stream.read_to_string(&mut msg).is_err() {
                        continue;
                    }

                    // a third chance to give up
                    let mut request: Request = match serde_json::from_str(&msg) {
                        Ok(req) => req,
                        Err(_) => continue,
                    };

                    #[cfg(feature = "log")]
                    log::info!("{discrim:?} recieved {request:?}");

                    // modify requests
                    match request.content_mut() {
                        RequestContent::Watch { label } => {
                            // convert a Watch into a WatchInternal
                            *request.content_mut() = RequestContent::WatchInternal(WatchInternal {
                                label: std::mem::take(label),
                                sender: responder.clone(),
                                watcher: discrim.clone(),
                            })
                        }
                        RequestContent::ConfirmRecieve { id, pass } => {
                            // if the request is a confirmation to a response
                            // then confirm the response and unblock the self.pass() thing
                            // by sending a message
                            let confirm_handles = confirm_handles.clone();
                            let id = *id;
                            let pass = *pass;
                            tokio::spawn(async move {
                                if let Entry::Occupied(entry) =
                                    confirm_handles.lock().await.entry(id)
                                {
                                    let _ = entry.remove_entry().1.send(pass);
                                }
                            });
                            continue;
                        }
                        RequestContent::Subscribe {
                            channel,
                            priority,
                            component: _,
                        } => {
                            // first add the channel to self as a record
                            // and send a register event to the master space
                            // which is eventually get sent to the parent space
                            // and get added as into the passes
                            *request.content_mut() = RequestContent::Subscribe {
                                channel: channel.clone(),
                                priority: *priority,
                                component: Some(discrim.clone()),
                            };
                            *request.target_mut() = discrim.clone().immediate_parent().unwrap();
                            let _ = responder.send(Response::new_with_request(
                                ResponseContent::Success {
                                    content: ResponseSuccess::SubscribeAdded,
                                },
                                *request.id(),
                            ));
                        }
                        RequestContent::Unsubscribe {
                            channel,
                            component: _,
                        } => {
                            // first add the channel to self as a record
                            // and send a register event to the master space
                            // which is eventually get sent to the parent space
                            // and get added as into the passes
                            *request.content_mut() = RequestContent::Unsubscribe {
                                channel: channel.clone(),
                                component: Some(discrim.clone()),
                            };
                            *request.target_mut() = discrim.clone().immediate_parent().unwrap();
                        }
                        RequestContent::SetSocket { path } => {
                            // these requests goes to self
                            let _ = responder.send(Response::new_with_request(
                                ResponseContent::SetSocket(storage.path().join(path)),
                                *request.id(),
                            ));

                            let _ = responder.send(Response::new_with_request(
                                ResponseContent::Success {
                                    // this can never fail
                                    content: ResponseSuccess::ListenerSet {
                                        discrim: discrim.clone(),
                                    },
                                },
                                *request.id(), // this is a response
                            ));
                            continue;
                        }
                        RequestContent::GetState { label } => {
                            // get state of self
                            // will implement getting state of other components
                            // TODO
                            let val = match label {
                                StateValue::Focused => {
                                    serde_json::to_value(FOCUSED.get().unwrap()).unwrap()
                                }
                                StateValue::IsFocused => serde_json::to_value(
                                    FOCUSED.get().unwrap().lock().unwrap().starts_with(&discrim),
                                )
                                .unwrap(),
                                StateValue::TermSize => {
                                    #[derive(serde::Serialize)]
                                    struct TermSize {
                                        x: u32,
                                        y: u32,
                                    }
                                    let (x, y) = termion::terminal_size().unwrap();
                                    serde_json::to_value(TermSize {
                                        x: x as u32,
                                        y: y as u32,
                                    })
                                    .unwrap()
                                }
                                StateValue::WorkingDir => {
                                    serde_json::to_value(env::current_dir().unwrap()).unwrap()
                                }
                            };

                            let _ = responder.send(Response::new_with_request(
                                ResponseContent::Success {
                                    content: ResponseSuccess::Value { value: val },
                                },
                                *request.id(),
                            ));

                            continue;
                        }
                        RequestContent::Drop { discrim: to_drop } => {
                            // this goes to parent space
                            let to_drop = to_drop.as_ref().unwrap_or(&discrim).clone();
                            *request.target_mut() = to_drop.clone().immediate_parent().unwrap();
                            *request.content_mut() = RequestContent::Drop {
                                discrim: Some(to_drop),
                            };
                        }
                        RequestContent::Render { .. } => {
                            // this goes to master space
                            *request.target_mut() = Discriminator::master()
                        }
                        RequestContent::Spawn { .. } => {
                            // this goes to master space only when target is not specified
                            if request.target().is_empty() {
                                *request.target_mut() = discrim.clone().immediate_parent().unwrap();
                            }
                        }
                        RequestContent::Unwatch { watcher, .. } => *watcher = discrim.clone(),
                        // mark self as sender
                        RequestContent::Message { sender, target, .. } => {
                            *sender = discrim.clone();

                            // or else it will crash for this bad request
                            if target.is_empty() {
                                *target = Discriminator::master()
                            }
                        }
                        RequestContent::NewSpace { .. }
                        | RequestContent::FocusAt
                        | RequestContent::Suppress { .. }
                        | RequestContent::Unsuppress { .. }
                        | RequestContent::SetEntry { .. }
                        | RequestContent::RemoveEntry { .. }
                        | RequestContent::GetEntry { .. } => {}
                        RequestContent::WatchInternal(_) => unreachable!("boom"),
                    }

                    let responder = responder.clone();
                    tokio::task::spawn(async move {
                        // otherwise, the request gets sended to the master space
                        // and starts propagating downwards
                        let res = request.send().await;

                        // send a response to the request
                        // but requires no confirmation
                        // because the response is already a sort of confirmation
                        let _ = responder.send(res);
                    });
                }
            })
        };

        let _ = set_socket_recv.await;

        Ok(Self {
            child,
            label,
            storage,
            pool: Arc::new(Mutex::new(Pool::new(discrim.clone()))),
            discrim,
            command: [command].into_iter().chain(args).collect(),
            listener,
            responder,
            res: responder_send,
            confirm_handles,
            suppressors: Arc::new(Mutex::new(Suppressors::default())),
        })
    }

    pub async fn handle(&self, packet: &mut Packet<Request, Response>) {
        match packet.get().content() {
            RequestContent::Suppress { channel, priority } => {
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::Suppressed {
                            id: self
                                .suppressors
                                .lock()
                                .await
                                .insert(channel.clone(), *priority),
                        },
                    },
                    *packet.get().id(),
                ));
            }
            RequestContent::Unsuppress { channel, id } => {
                self.suppressors.lock().await.remove(channel.clone(), *id);
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::Unsuppressed,
                    },
                    *packet.get().id(),
                ));
            }
            RequestContent::RemoveEntry { label } => {
                self.pool.lock().await.remove(label, &self.discrim);
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::RemovedValue,
                    },
                    *packet.get().id(),
                ));
            }
            RequestContent::Message {
                content,
                sender,
                target,
                tag,
            } => {
                let mut event = Event::Message {
                    sender: sender.clone(),
                    target: target.clone(),
                    content: content.clone(),
                    tag: tag.clone(),
                };
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::MessageDelivered,
                    },
                    *packet.get().id(),
                ));
                // unwraps the request, and pass to self as an event
                // which will then get sent to the client as a normal event
                let _ = self.pass(&mut event, None).await;
            }
            // spawn should be passed to spaces, no processes
            RequestContent::Spawn { .. } => {
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Undelivered,
                    *packet.get().id(),
                ));
            }
            RequestContent::GetEntry { label } => {
                // just return an entry from pool, nothing special
                let res = match self.pool.lock().await.get(label) {
                    Some(value) => ResponseContent::Success {
                        content: ResponseSuccess::Value { value },
                    },
                    None => ResponseContent::Error {
                        content: ResponseError::EntryNotFound,
                    },
                };
                let _ = packet.respond(Response::new_with_request(res, *packet.get().id()));
            }
            RequestContent::SetEntry { label, value } => {
                // set an entry, this never fails
                self.pool.lock().await.set(label, value.clone());
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::ValueSet,
                    },
                    *packet.get().id(),
                ));
            }
            RequestContent::WatchInternal(watch) => {
                // add a watcher to an entry
                self.pool.lock().await.watch(
                    watch.label.clone(),
                    watch.sender.clone(),
                    watch.watcher.clone(),
                );
                let _ = packet.respond(Response::new_with_request(
                    ResponseContent::Success {
                        content: ResponseSuccess::Watching,
                    },
                    *packet.get().id(),
                ));
            }
            RequestContent::Unwatch { label, watcher } => {
                // remove watcher from entry
                let res = if self
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
                let _ = packet.respond(Response::new_with_request(res, *packet.get().id()));
            }
            // confirmreceive gets filtered out and handles in the listener loop
            // so we will never get it
            RequestContent::ConfirmRecieve { .. }
            | RequestContent::Watch { .. }
            | RequestContent::Unsubscribe { .. }
            | RequestContent::Drop { .. }
            | RequestContent::Subscribe { .. }
            | RequestContent::SetSocket { .. }
            | RequestContent::NewSpace { .. }
            | RequestContent::FocusAt
            | RequestContent::GetState { .. }
            | RequestContent::Render { .. } => {
                unreachable!("not a real request")
            }
        }
    }

    /// send a response and wait for confirmation
    pub async fn send_event(&self, resp: Response) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();
        self.confirm_handles.lock().await.insert(resp.id(), tx);
        self.res.send(resp).unwrap();
        rx
    }

    pub async fn suppress_level(&self, channels: &[Subscription]) -> Option<u32> {
        self.suppressors.lock().await.suppress_level(channels)
    }
}

#[async_trait]
impl Component for Process {
    fn label(&self) -> &str {
        &self.label
    }

    fn discrim(&self) -> &Discriminator {
        &self.discrim
    }

    fn storage(&self) -> &Storage {
        &self.storage
    }

    async fn pass(&self, event: &mut Event, _suppress_level: Option<u32>) -> Unevaluated<bool> {
        #[cfg(feature = "log")]
        log::debug!("{:?} got event {event:?}", self.discrim);
        // requestpacket is a request, not an event in a real sense
        // and it doesnt serialise into EventSerde either
        // so best just handle it out and filter it first
        if let Event::RequestPacket(packet) = event {
            self.handle(packet).await;
            return false.into();
        }

        let resp = Response::new(ResponseContent::Event {
            content: EventSerde::from_event(event),
        });

        let rx = self.send_event(resp).await;
        // dont block
        // or else it will keep parent.processes locked
        Unevaluated::Unevaluated(tokio::spawn(async move { rx.await.unwrap_or(true) }))
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.responder.abort();
        self.listener.abort();
    }
}
