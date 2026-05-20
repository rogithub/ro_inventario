use uuid::Uuid;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct ClienteBuscado {
    pub id: Uuid,
    pub nombre: String,
    pub telefono: String,
    pub email: String,
}
