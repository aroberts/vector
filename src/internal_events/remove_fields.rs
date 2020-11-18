use super::InternalEvent;
use crate::event::Lookup;
use metrics::counter;

#[derive(Debug)]
pub struct RemoveFieldsEventProcessed;

impl InternalEvent for RemoveFieldsEventProcessed {
    fn emit_metrics(&self) {
        counter!("events_processed_total", 1);
    }
}

#[derive(Debug)]
pub struct RemoveFieldsFieldMissing<'a> {
    pub field: Lookup<'a>,
}

impl<'a> InternalEvent for RemoveFieldsFieldMissing<'a> {
    fn emit_logs(&self) {
        debug!(message = "Field did not exist.", field = %self.field, rate_limit_secs = 30);
    }
}
