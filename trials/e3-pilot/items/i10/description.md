Reject changes that do nothing. `apply` now fails, leaving the inventory
unchanged, with a new `ApplyError::ZeroQuantity(sku)` for a `Restock` or
`Sell` of 0 units, and with a new `ApplyError::AlreadyDiscontinued(sku)`
for a `Discontinue` of an item that is already discontinued.
