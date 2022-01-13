use tracing::info;
use crate::util::bot_context::BotContext;

#[derive(Clone, Eq, PartialEq)]
pub enum BotStatus {
    STARTING,
    STANDBY,
    PRIMARY,
    TERMINATING
}

impl BotStatus {
    pub fn name(&self) -> &str {
        match self {
            BotStatus::STARTING => "STARTING",
            BotStatus::STANDBY => "STANDBY",
            BotStatus::PRIMARY => "PRIMARY",
            BotStatus::TERMINATING => "TERMINATING",
        }
    }
}

impl BotContext {
    pub fn set_status(&self, new_status: BotStatus) {
        // get lock
        let mut status = self.status.write();

        info!("Cluster status change: {} => {}", status.name(), new_status.name());

        // update metrics
        self.metrics.status.reset();
        self.metrics.status.get_metric_with_label_values(&[new_status.name()]).unwrap().set(1);

        //store new status
        *status = new_status;
    }

    pub fn is_status(&self, status: BotStatus) -> bool {
        *self.status.read() == status
    }
}