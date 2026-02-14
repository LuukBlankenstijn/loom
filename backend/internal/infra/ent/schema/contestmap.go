package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/field"
)

// ContestMap holds the schema definition for the ContestMap entity.
type ContestMap struct {
	ent.Schema
}

// Fields of the ContestMap.
func (ContestMap) Fields() []ent.Field {
	return []ent.Field{
		field.String("contest_id").Unique().Immutable(),
		field.Int("map_id"),
	}
}

// Edges of the ContestMap.
func (ContestMap) Edges() []ent.Edge {
	return nil
}
