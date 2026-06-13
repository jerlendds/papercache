#[derive(Debug, Clone)]
pub enum IndexCommand {
    UpsertDocument {
        document_id: String,
    },
    DeleteDocument {
        document_id: String,
    },
    #[allow(dead_code)]
    RebuildAll,
}
