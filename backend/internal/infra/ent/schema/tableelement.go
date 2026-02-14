package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"github.com/google/uuid"
)

// TableElement holds the schema definition for the TableElement entity.
type TableElement struct {
	ent.Schema
}

// Fields of the TableElement.
func (TableElement) Fields() []ent.Field {
	return []ent.Field{
		field.UUID("id", uuid.UUID{}).Immutable(),
		field.Int("x"),
		field.Int("y"),
		field.Enum("rotation").
			Values("0", "90", "180", "270").
			Default("0"),
	}
}

// Edges of the TableElement.
func (TableElement) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("map", ContestAreaMap.Type).Ref("tables").Unique(),
		edge.To("station", Station.Type).
			Unique(),
	}
}
