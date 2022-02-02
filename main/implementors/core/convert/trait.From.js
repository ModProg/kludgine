(function() {var implementors = {};
implementors["kludgine"] = [{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;&amp;'a mut <a class=\"struct\" href=\"kludgine/core/prelude/struct.Sprite.html\" title=\"struct kludgine::core::prelude::Sprite\">Sprite</a>&gt; for <a class=\"enum\" href=\"kludgine/tilemap/enum.TileSprite.html\" title=\"enum kludgine::tilemap::TileSprite\">TileSprite</a>&lt;'a&gt;","synthetic":false,"types":["kludgine::tilemap::TileSprite"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine/core/prelude/struct.SpriteSource.html\" title=\"struct kludgine::core::prelude::SpriteSource\">SpriteSource</a>&gt; for <a class=\"enum\" href=\"kludgine/tilemap/enum.TileSprite.html\" title=\"enum kludgine::tilemap::TileSprite\">TileSprite</a>&lt;'a&gt;","synthetic":false,"types":["kludgine::tilemap::TileSprite"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine/core/prelude/struct.Sprite.html\" title=\"struct kludgine::core::prelude::Sprite\">Sprite</a>&gt; for <a class=\"enum\" href=\"kludgine/tilemap/enum.PersistentTileSource.html\" title=\"enum kludgine::tilemap::PersistentTileSource\">PersistentTileSource</a>","synthetic":false,"types":["kludgine::tilemap::PersistentTileSource"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine/core/prelude/struct.SpriteSource.html\" title=\"struct kludgine::core::prelude::SpriteSource\">SpriteSource</a>&gt; for <a class=\"enum\" href=\"kludgine/tilemap/enum.PersistentTileSource.html\" title=\"enum kludgine::tilemap::PersistentTileSource\">PersistentTileSource</a>","synthetic":false,"types":["kludgine::tilemap::PersistentTileSource"]}];
implementors["kludgine_app"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"kludgine_core/error/enum.Error.html\" title=\"enum kludgine_core::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"kludgine_app/prelude/enum.Error.html\" title=\"enum kludgine_app::prelude::Error\">Error</a>","synthetic":false,"types":["kludgine_app::error::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.53/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt; for <a class=\"enum\" href=\"kludgine_app/prelude/enum.Error.html\" title=\"enum kludgine_app::prelude::Error\">Error</a>","synthetic":false,"types":["kludgine_app::error::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_app/prelude/struct.WindowBuilder.html\" title=\"struct kludgine_app::prelude::WindowBuilder\">WindowBuilder</a>&gt; for WinitWindowBuilder","synthetic":false,"types":["winit::window::WindowBuilder"]}];
implementors["kludgine_core"] = [{"text":"impl&lt;U:&nbsp;<a class=\"trait\" href=\"https://docs.rs/palette/0.6.0/palette/palette/component/trait.Component.html\" title=\"trait palette::component::Component\">Component</a> + <a class=\"trait\" href=\"https://docs.rs/palette/0.6.0/palette/palette/component/trait.IntoComponent.html\" title=\"trait palette::component::IntoComponent\">IntoComponent</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.58.1/std/primitive.f32.html\">f32</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/palette/0.6.0/palette/palette/alpha/alpha/struct.Alpha.html\" title=\"struct palette::alpha::alpha::Alpha\">Alpha</a>&lt;<a class=\"struct\" href=\"https://docs.rs/palette/0.6.0/palette/palette/rgb/rgb/struct.Rgb.html\" title=\"struct palette::rgb::rgb::Rgb\">Rgb</a>&lt;<a class=\"struct\" href=\"https://docs.rs/palette/0.6.0/palette/palette/encoding/srgb/struct.Srgb.html\" title=\"struct palette::encoding::srgb::Srgb\">Srgb</a>, U&gt;, U&gt;&gt; for <a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>","synthetic":false,"types":["kludgine_core::color::Color"]},{"text":"impl&lt;U:&nbsp;<a class=\"trait\" href=\"https://docs.rs/palette/0.6.0/palette/palette/component/trait.Component.html\" title=\"trait palette::component::Component\">Component</a> + <a class=\"trait\" href=\"https://docs.rs/palette/0.6.0/palette/palette/component/trait.IntoComponent.html\" title=\"trait palette::component::IntoComponent\">IntoComponent</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.58.1/std/primitive.f32.html\">f32</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/palette/0.6.0/palette/palette/rgb/rgb/struct.Rgb.html\" title=\"struct palette::rgb::rgb::Rgb\">Rgb</a>&lt;<a class=\"struct\" href=\"https://docs.rs/palette/0.6.0/palette/palette/encoding/srgb/struct.Srgb.html\" title=\"struct palette::encoding::srgb::Srgb\">Srgb</a>, U&gt;&gt; for <a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>","synthetic":false,"types":["kludgine_core::color::Color"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>&gt; for <a class=\"type\" href=\"https://docs.rs/palette/0.6.0/palette/palette/rgb/type.Srgba.html\" title=\"type palette::rgb::Srgba\">Srgba</a>","synthetic":false,"types":["palette::rgb::Srgba"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>&gt; for Rgba","synthetic":false,"types":["easygpu::color::Rgba"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;Rgba&gt; for <a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>","synthetic":false,"types":["kludgine_core::color::Color"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_core/prelude/struct.Color.html\" title=\"struct kludgine_core::prelude::Color\">Color</a>&gt; for Rgba8","synthetic":false,"types":["easygpu::color::Rgba8"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;ImageError&gt; for <a class=\"enum\" href=\"kludgine_core/enum.Error.html\" title=\"enum kludgine_core::Error\">Error</a>","synthetic":false,"types":["kludgine_core::error::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;Error&gt; for <a class=\"enum\" href=\"kludgine_core/enum.Error.html\" title=\"enum kludgine_core::Error\">Error</a>","synthetic":false,"types":["kludgine_core::error::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;BufferAsyncError&gt; for <a class=\"enum\" href=\"kludgine_core/enum.Error.html\" title=\"enum kludgine_core::Error\">Error</a>","synthetic":false,"types":["kludgine_core::error::Error"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.58.1/alloc/sync/struct.Arc.html\" title=\"struct alloc::sync::Arc\">Arc</a>&lt;<a class=\"struct\" href=\"kludgine_core/scene/struct.Scene.html\" title=\"struct kludgine_core::scene::Scene\">Scene</a>&gt;&gt; for <a class=\"struct\" href=\"kludgine_core/scene/struct.Target.html\" title=\"struct kludgine_core::scene::Target\">Target</a>","synthetic":false,"types":["kludgine_core::scene::Target"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_core/scene/struct.Scene.html\" title=\"struct kludgine_core::scene::Scene\">Scene</a>&gt; for <a class=\"struct\" href=\"kludgine_core/scene/struct.Target.html\" title=\"struct kludgine_core::scene::Target\">Target</a>","synthetic":false,"types":["kludgine_core::scene::Target"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"kludgine_core/shape/enum.PathEvent.html\" title=\"enum kludgine_core::shape::PathEvent\">PathEvent</a>&lt;Pixels&gt;&gt; for LyonPathEvent","synthetic":false,"types":["lyon_path::events::PathEvent"]},{"text":"impl&lt;S, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; for <a class=\"struct\" href=\"kludgine_core/shape/struct.Path.html\" title=\"struct kludgine_core::shape::Path\">Path</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/iter/traits/collect/trait.IntoIterator.html\" title=\"trait core::iter::traits::collect::IntoIterator\">IntoIterator</a>&lt;Item = <a class=\"enum\" href=\"kludgine_core/shape/enum.PathEvent.html\" title=\"enum kludgine_core::shape::PathEvent\">PathEvent</a>&lt;S&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["kludgine_core::shape::path::Path"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kludgine_core/sprite/struct.SpriteAnimations.html\" title=\"struct kludgine_core::sprite::SpriteAnimations\">SpriteAnimations</a>&gt; for <a class=\"struct\" href=\"kludgine_core/sprite/struct.Sprite.html\" title=\"struct kludgine_core::sprite::Sprite\">Sprite</a>","synthetic":false,"types":["kludgine_core::sprite::Sprite"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;VMetrics&gt; for <a class=\"struct\" href=\"kludgine_core/text/prepared/struct.VMetrics.html\" title=\"struct kludgine_core::text::prepared::VMetrics\">VMetrics</a>","synthetic":false,"types":["kludgine_core::text::prepared::VMetrics"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.58.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;&amp;'a DynamicImage&gt; for <a class=\"struct\" href=\"kludgine_core/texture/struct.Texture.html\" title=\"struct kludgine_core::texture::Texture\">Texture</a>","synthetic":false,"types":["kludgine_core::texture::Texture"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()