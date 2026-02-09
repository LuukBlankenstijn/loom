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
		field.Bytes("image_data"),
		field.String("contest_id").Unique().Immutable(),
	}
}

// Edges of the Wallpaper.
func (Wallpaper) Edges() []ent.Edge {
	return nil
}
