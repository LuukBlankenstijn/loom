package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

// ContestAreaMap holds the schema definition for the ContestAreaMap entity.
type ContestAreaMap struct {
	ent.Schema
}

// Fields of the ContestAreaMap.
func (ContestAreaMap) Fields() []ent.Field {
	return []ent.Field{
		field.String("Name").Immutable(),
	}
}

// Edges of the ContestAreaMap.
func (ContestAreaMap) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("doors", DoorElement.Type),
		edge.To("walls", WallElement.Type),
		edge.To("tables", TableElement.Type),
	}
}
