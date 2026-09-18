use sky_server::shared::remote::SpawnTask;

#[derive(Clone)]
pub struct LeptosSpawnTask;
impl SpawnTask for LeptosSpawnTask {
    fn spawn_task(future: impl Future<Output = ()> + 'static) {
        leptos::task::spawn_local(future);
    }
}
