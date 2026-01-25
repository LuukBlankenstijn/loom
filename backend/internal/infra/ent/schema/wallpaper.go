package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/field"
)

// Wallpaper holds the schema definition for the Wallpaper entity.
type Wallpaper struct {
	ent.Schema
}

// Fields of the Wallpaper.
func (Wallpaper) Fields() []ent.Field {
	return []ent.Field{
		field.Bytes("image_data").Optional().Nillable(),
		field.String("contest_id").Immutable(),
	}
}

// Edges of the Wallpaper.
func (Wallpaper) Edges() []ent.Edge {
	return nil
}
