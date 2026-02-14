package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"github.com/google/uuid"
)

// DoorElement holds the schema definition for the DoorElement entity.
type DoorElement struct {
	ent.Schema
}

// Fields of the DoorElement.
func (DoorElement) Fields() []ent.Field {
	return []ent.Field{
		field.UUID("id", uuid.UUID{}).Immutable(),
		field.Int("x"),
		field.Int("y"),
		field.Enum("rotation").
			Values("0", "90", "180", "270").
			Default("0"),
	}
}

// Edges of the DoorElement.
func (DoorElement) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("map", ContestAreaMap.Type).Ref("doors").Unique(),
	}
}
