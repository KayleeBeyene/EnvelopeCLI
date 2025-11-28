# EnvelopeCLI Data Format

This document describes the JSON data format used by EnvelopeCLI. Understanding this format allows for manual editing, backup inspection, and custom tooling.

## Data Location

By default, EnvelopeCLI stores data in:

- **Linux/macOS:** `~/.envelope/`
- **Windows:** `%APPDATA%\envelope\`

## Directory Structure

```
~/.envelope/
├── config.json          # User settings
├── data/
│   ├── accounts.json    # Account definitions
│   ├── budget.json      # Categories, groups, allocations
│   ├── transactions.json # All transactions
│   └── payees.json      # Payee list with rules
├── audit.log            # Append-only change log
└── backups/             # Automatic backups
```

## Schema Version

All data files include a `schema_version` field to support future migrations:

```json
{
  "schema_version": 1,
  ...
}
```

---

## config.json

User preferences and settings.

```json
{
  "schema_version": 1,
  "budget_period_type": "monthly",
  "encryption_enabled": false,
  "encryption": {
    "enabled": false,
    "key_params": null,
    "verification_hash": null
  },
  "backup_retention": {
    "daily_count": 30,
    "monthly_count": 12
  },
  "currency_symbol": "$",
  "date_format": "%Y-%m-%d",
  "first_day_of_week": 0,
  "setup_completed": true
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `budget_period_type` | string | `"monthly"`, `"weekly"`, or `"biweekly"` |
| `encryption_enabled` | boolean | Whether encryption is enabled |
| `backup_retention.daily_count` | integer | Number of daily backups to keep |
| `backup_retention.monthly_count` | integer | Number of monthly backups to keep |
| `currency_symbol` | string | Currency symbol for display |
| `date_format` | string | strftime format for dates |
| `first_day_of_week` | integer | 0 = Sunday, 1 = Monday |

---

## accounts.json

List of all accounts.

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Chase Checking",
    "type": "checking",
    "on_budget": true,
    "archived": false,
    "starting_balance": 100000,
    "notes": "Primary checking account",
    "last_reconciled_date": "2025-01-15",
    "last_reconciled_balance": 250000,
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-15T12:00:00Z",
    "sort_order": 0
  }
]
```

### Account Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | string | Display name |
| `type` | string | `checking`, `savings`, `credit`, `cash`, `investment`, `lineofcredit`, `other` |
| `on_budget` | boolean | Whether included in budget |
| `archived` | boolean | Soft-deleted |
| `starting_balance` | integer | Initial balance in cents |
| `notes` | string | Optional notes |
| `last_reconciled_date` | date | Last reconciliation date (YYYY-MM-DD) |
| `last_reconciled_balance` | integer | Balance at last reconciliation (cents) |
| `created_at` | datetime | Creation timestamp (ISO 8601) |
| `updated_at` | datetime | Last modification timestamp |
| `sort_order` | integer | Display order |

---

## budget.json

Categories, groups, and budget allocations.

```json
{
  "schema_version": 1,
  "groups": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "Bills",
      "sort_order": 0
    }
  ],
  "categories": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "name": "Rent",
      "group_id": "550e8400-e29b-41d4-a716-446655440001",
      "sort_order": 0,
      "hidden": false
    }
  ],
  "allocations": [
    {
      "category_id": "550e8400-e29b-41d4-a716-446655440002",
      "period": "2025-01",
      "budgeted": 150000,
      "carryover": 0
    }
  ]
}
```

### Category Group Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | string | Group name |
| `sort_order` | integer | Display order |

### Category Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | string | Category name |
| `group_id` | UUID | Parent group ID |
| `sort_order` | integer | Order within group |
| `hidden` | boolean | Whether hidden from view |

### Allocation Fields

| Field | Type | Description |
|-------|------|-------------|
| `category_id` | UUID | Category this allocation is for |
| `period` | string | Budget period (e.g., "2025-01") |
| `budgeted` | integer | Amount budgeted in cents |
| `carryover` | integer | Rolled over from previous period (cents) |

---

## transactions.json

All transactions across all accounts.

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440003",
    "account_id": "550e8400-e29b-41d4-a716-446655440000",
    "date": "2025-01-15",
    "amount": -5000,
    "payee_id": null,
    "payee_name": "Coffee Shop",
    "category_id": "550e8400-e29b-41d4-a716-446655440004",
    "splits": [],
    "memo": "Morning coffee",
    "status": "cleared",
    "transfer_transaction_id": null,
    "import_id": null,
    "created_at": "2025-01-15T08:30:00Z",
    "updated_at": "2025-01-15T08:30:00Z"
  }
]
```

### Transaction Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `account_id` | UUID | Account this belongs to |
| `date` | date | Transaction date (YYYY-MM-DD) |
| `amount` | integer | Amount in cents (negative = outflow) |
| `payee_id` | UUID? | Optional payee reference |
| `payee_name` | string | Payee display name |
| `category_id` | UUID? | Category (null for splits/transfers) |
| `splits` | array | Split transactions |
| `memo` | string | Optional memo |
| `status` | string | `pending`, `cleared`, or `reconciled` |
| `transfer_transaction_id` | UUID? | Linked transfer transaction |
| `import_id` | string? | Import deduplication ID |
| `created_at` | datetime | Creation timestamp |
| `updated_at` | datetime | Last modification timestamp |

### Split Fields

```json
{
  "category_id": "550e8400-e29b-41d4-a716-446655440005",
  "amount": -3000,
  "memo": "Groceries portion"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `category_id` | UUID | Category for this split |
| `amount` | integer | Amount in cents |
| `memo` | string | Optional memo |

---

## payees.json

Payee list with auto-categorization rules.

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440006",
    "name": "Grocery Store",
    "default_category_id": "550e8400-e29b-41d4-a716-446655440007",
    "transaction_count": 15,
    "last_used": "2025-01-15"
  }
]
```

### Payee Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier |
| `name` | string | Payee name |
| `default_category_id` | UUID? | Auto-categorization default |
| `transaction_count` | integer | Number of transactions |
| `last_used` | date | Last transaction date |

---

## Money Representation

All monetary values are stored as integers representing **cents** (or the smallest currency unit):

- `$10.00` = `1000`
- `$100.50` = `10050`
- `-$25.99` = `-2599`

This avoids floating-point precision issues.

---

## Date Formats

- **Date:** `YYYY-MM-DD` (e.g., `2025-01-15`)
- **DateTime:** ISO 8601 (e.g., `2025-01-15T08:30:00Z`)
- **Period:** `YYYY-MM` for monthly (e.g., `2025-01`)

---

## Editing Data Files

You can manually edit JSON files, but:

1. **Always create a backup first:** `envelope backup create`
2. **Maintain referential integrity:** IDs must match across files
3. **Validate JSON syntax:** Use a JSON validator
4. **Restart after editing:** Changes won't be seen until reload

### Example: Add a Category Manually

1. Open `~/.envelope/data/budget.json`
2. Find the `categories` array
3. Add a new category object with a unique UUID
4. Save the file
5. Restart EnvelopeCLI or reload data

---

## Backup Format

Backups are stored as timestamped JSON files:

```
~/.envelope/backups/
├── 2025-01-15_120000.json
├── 2025-01-14_120000.json
└── ...
```

Each backup contains the complete state:

```json
{
  "created_at": "2025-01-15T12:00:00Z",
  "config": { ... },
  "accounts": [ ... ],
  "categories": [ ... ],
  "transactions": [ ... ],
  "payees": [ ... ]
}
```
