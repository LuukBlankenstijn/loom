package schema

import (
	"regexp"
	"time"

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
		field.String("mime_type"),
		field.String("color").
			Match(regexp.MustCompile(`^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$`)).
			Default("#ffffff").
			Comment("Hexadecimal color value"),
		field.Time("updated_at").Default(time.Now).UpdateDefault(time.Now),
		field.String("contest_id").Unique().Immutable(),
	}
}

// Edges of the Wallpaper.
func (Wallpaper) Edges() []ent.Edge {
	return nil
}
