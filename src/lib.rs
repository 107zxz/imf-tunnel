#![allow(unused)]
use gdnative::{prelude::*, tasks::{Async, AsyncMethod, Spawner}};
use tokio::{task::LocalSet, runtime::{Builder, Runtime}};
use std::time::Duration;
use localtunnel::{open_tunnel, broadcast};
use std::sync::mpsc::sync_channel;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {

    gdnative::tasks::register_runtime(&handle);
    gdnative::tasks::set_executor(EXECUTOR.with(|e| *e));

    handle.add_class::<AsyncMethods>();
    handle.add_class::<AsyncExecutorDriver>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);


// Tunnel Stuff
// #[derive(NativeClass)]
// #[inherit(Node)]
// pub struct Tunnel;

// impl Tunnel {
//     fn new(_base: &Node) -> Self {
//         Tunnel
//     }
// }

// #[methods]
// impl Tunnel {

//     #[method]
//     fn _ready(&self, #[base] base: &Node) {
//         godot_print!("Sussy BAKA!");
//     }

// }

// Async stuff
thread_local! {
    static EXECUTOR: &'static SharedLocalPool = {
        Box::leak(Box::new(SharedLocalPool::default()))
    };
}

#[derive(Default)]
struct SharedLocalPool {
    local_set: LocalSet,
}
impl futures::task::LocalSpawn for SharedLocalPool {
    fn spawn_local_obj(&self, future: futures::task::LocalFutureObj<'static, ()>) -> Result<(), futures::task::SpawnError> {
        self.local_set.spawn_local(future);

        Ok(())
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
struct AsyncExecutorDriver {
    runtime: Runtime,
}
impl AsyncExecutorDriver {
    fn new(_base: &Node) -> Self {
        AsyncExecutorDriver {
            runtime: Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap(),
        }
    }
}

#[methods]
impl AsyncExecutorDriver {
    #[method]
    fn _process(&self, #[base] _base: &Node, _delta: f64) {
        EXECUTOR.with(|e| {
            self.runtime
                .block_on(async {
                    e.local_set
                        .run_until(async {
                            tokio::task::spawn_local(async {}).await
                        })
                        .await
                }).unwrap()
        })
    }
}


// More test copying
#[derive(NativeClass)]
#[inherit(Reference)]
#[register_with(register_methods)]
struct AsyncMethods {
    a: i32
}

#[methods]
impl AsyncMethods {
    fn new(_owner: TRef<Reference>) -> Self {
        AsyncMethods {
            a: 4
        }
    }
}

struct TunnelFn;

impl AsyncMethod<AsyncMethods> for TunnelFn {
    fn spawn_with(&self, spawner: Spawner<'_, AsyncMethods>) {
        spawner.spawn(|ctx, _this, mut args| {
            // let a = args.read::<i32>().get().unwrap();

            let obj = args.read::<Ref<Object>>().get().unwrap();
            let subdom = args.read::<GodotString>().get().unwrap();

            async move {

                /*
                    This is identical to var b: i32 = yield()

                    ```rust
                    let b = ctx.until_resume().await;
                    let b = i32::from_variant(&b).unwrap();
                    ```
                */

                /*
                    Sets c to the return value of the passed function name called on the passed object.
                    As the passed function is async, this sets it to a GDScriptFunctionState.

                    let c = unsafe { obj.assume_safe().call(name, &[]) };
                    let c:Ref<Reference> = Ref::<Reference>::from_variant(&c).unwrap();
                    let c:TRef<Reference> = unsafe { c.assume_safe() };

                    Equivalent to c = yield(c, "completed"), which waits for the async func to finish and takes the return value. 
                    The assert statement is to ensure c only returns one value

                    let c = ctx.signal(c, "completed").unwrap().await;
                    assert_eq!(1, c.len());
                    let c = i32::from_variant(&c[0]).unwrap();
                    println!("c: {}", c);
                */

                println!("Before tunneling");

                let (notify_shutdown, _) = broadcast::channel(1);
                let result = open_tunnel(
                    Some("https://localtunnel.me"),
                    Some(&subdom.to_string()),
                    Some("localhost"),
                    10567,
                    notify_shutdown.clone(),
                    10,
                )
                .await
                .unwrap();

                println!("Tunneled to: {}", result);

                let pr = result.to_variant();

                // Pass url to godot
                let c = unsafe { obj.assume_safe().call("tunnel_callback", &[pr]) };
                // let c:Ref<Reference> = Ref::<Reference>::from_variant(&c).unwrap();
                // let c:TRef<Reference> = unsafe { c.assume_safe() };

                // Yield until resumed (to close the tunnel)
                ctx.until_resume().await;

                println!("About to kill");

                // Shutdown the background tasks by sending a signal.
                let _ = notify_shutdown.send(());

                println!("Tunnel killed");

                ().to_variant()
            }

            // let obj = args.read::<Ref<Object>>().get().unwrap();

            // async move {

            //     // let b = ctx.until_resume().await;
            //     // let b = i32::from_variant(&b).unwrap();

            //     // Emit signal
            //     let c = unsafe { obj.assume_safe() };
            //     // let c = Ref::<Reference>::from_variant(&c).unwrap();
            //     // let c = unsafe { c.assume_safe() };

            //     let c = ctx.signal(c, "completed").unwrap().await;
            //     assert_eq!(1, c.len());

            //     let c = i32::from_variant(&c[0]).unwrap();

            //     async_std::task::sleep(time::Duration::from_secs(4));

            //     // This has something to do with binding signals
            //     (c).to_variant()
            // }
        });
    }
}

fn register_methods(builder: &ClassBuilder<AsyncMethods>) {
    builder.method("open_tunnel", Async::new(TunnelFn)).done();
}