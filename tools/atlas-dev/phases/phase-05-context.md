# Phase 05: Documentation Context System (SQLite)

**Objective:** Aggregate phase context from DB + phase files.
**Priority:** MEDIUM | **Depends On:** Phase 4

## Deliverables
- `context current` - Full context for current phase (join phases + decisions + features)
- `context phase <path>` - Context for specific phase
- Combines DB data + phase file parsing (instructions)

## Key Query
```sql
SELECT
    p.*,
    c.completed as cat_completed,
    c.total as cat_total,
    (SELECT GROUP_CONCAT(id) FROM decisions WHERE component = p.category) as decisions
FROM phases p
JOIN categories c ON p.category = c.name
WHERE p.id = ?;
```

## Output
`context current`: ~200 tokens (provides all info AI needs to start work)
