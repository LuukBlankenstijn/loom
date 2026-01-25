package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// Contest holds the schema definition for the Contest entity.
type Contest struct {
	ent.Schema
}

// Fields of the Contest.
func (Contest) Fields() []ent.Field {
	return []ent.Field{
		field.String("id").
			NotEmpty().
			Unique().
			Immutable(),
		field.String("name"),
		field.Time("start_time"),
		field.Time("end_time"),
	}
}

// Edges of the Contest.
func (Contest) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("teams", Team.Type),
	}
}
