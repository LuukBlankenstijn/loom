package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"github.com/google/uuid"
)

// WallElement holds the schema definition for the WallElement entity.
type WallElement struct {
	ent.Schema
}

// Fields of the WallElement.
func (WallElement) Fields() []ent.Field {
	return []ent.Field{
		field.UUID("id", uuid.UUID{}).Immutable(),
		field.Int("x_start"),
		field.Int("y_start"),
		field.Int("x_end"),
		field.Int("y_end"),
	}
}

// Edges of the WallElement.
func (WallElement) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("map", ContestAreaMap.Type).Ref("walls").Unique(),
	}
}
