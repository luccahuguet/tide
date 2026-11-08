use std::json;
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
pub struct Inventory {
    pub warehouse: String,
    pub items: Vec<Item>,
}
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[codec(rename_all = "camelCase")]
pub struct Item {
    pub sku: String,
    pub quantity: u32,
    pub unit_price_cents: u64,
    #[codec(default)]
    pub tags: Vec<String>,
    pub discontinued: Option<bool>,
}
#[derive(Encode, Decode)]
pub enum Change {
    Restock { sku: String, qty: u32 },
    Sell { sku: String, qty: u32 },
    Discontinue { sku: String },
}
#[derive(Debug)]
pub enum LoadError {
    Json(json::Error),
    Duplicate(String),
}
#[derive(Debug)]
pub enum ApplyError {
    Json(json::Error),
    UnknownSku(String),
    Insufficient(String),
    Overflow(String),
}
pub fn load(bytes: &[u8]) -> Result<Inventory, LoadError> {
    let inv: Inventory = json::decode(bytes).map_err(LoadError::Json)?;
    let mut seen = std::collections::HashSet::new();
    for i in &inv.items {
        if !seen.insert(&i.sku) {
            return Err(LoadError::Duplicate(i.sku.clone()));
        }
    }
    Ok(inv)
}
pub fn apply(inv: &mut Inventory, changes: &[u8]) -> Result<(), ApplyError> {
    let changes: Vec<Change> = json::decode(changes).map_err(ApplyError::Json)?;
    let mut next = inv.clone();
    for change in changes {
        let sku = match &change {
            Change::Restock { sku, .. }
            | Change::Sell { sku, .. }
            | Change::Discontinue { sku } => sku,
        };
        let item = next
            .items
            .iter_mut()
            .find(|i| &i.sku == sku)
            .ok_or_else(|| ApplyError::UnknownSku(sku.clone()))?;
        match change {
            Change::Restock { qty, .. } => {
                item.quantity = item
                    .quantity
                    .checked_add(qty)
                    .ok_or_else(|| ApplyError::Overflow(item.sku.clone()))?
            }
            Change::Sell { qty, .. } => {
                item.quantity = item
                    .quantity
                    .checked_sub(qty)
                    .ok_or_else(|| ApplyError::Insufficient(item.sku.clone()))?
            }
            Change::Discontinue { .. } => item.discontinued = Some(true),
        }
    }
    *inv = next;
    Ok(())
}
pub fn save(inv: &Inventory) -> Vec<u8> {
    json::encode(inv)
}
