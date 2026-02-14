package schema

import (
	"time"

	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Station holds the schema definition for the Station entity.
type Station struct {
	ent.Schema
}

// Fields of the Station.
func (Station) Fields() []ent.Field {
	return []ent.Field{
		field.String("ip").Immutable().Unique(),
		field.Time("connected_at").Default(time.Now()),
		field.Time("disconnected_at").Optional().Nillable(),
	}
}

// Edges of the Station.
func (Station) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("table_element", TableElement.Type).
			Ref("station"),
	}
}
