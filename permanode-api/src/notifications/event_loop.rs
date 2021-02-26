use super::*;

#[async_trait]
impl<H: LauncherSender<PermanodeAPIBuilder<H>>> EventLoop<PermanodeAPISender<H>> for Notifications {
    async fn event_loop(
        &mut self,
        status: Result<(), Need>,
        supervisor: &mut Option<PermanodeAPISender<H>>,
    ) -> Result<(), Need> {
        todo!()
    }
}
