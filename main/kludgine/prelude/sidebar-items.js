initSidebarItems({"enum":[["AnimationMode","The animation mode of the sprite."],["ElementState","Describes the input state of a key."],["Error","All errors that kludgine-app can return."],["Event","An input Event"],["EventStatus","Whether an event has been processed or ignored."],["MouseButton","Describes a button of a mouse controller."],["MouseScrollDelta","Describes a difference in the mouse scroll wheel state."],["PathEvent","An entry in a [`Path`]."],["Text","Text rendering functionality"],["TouchPhase","Describes touch-screen input state."],["VirtualKeyCode","Symbolic name for a keyboard key."]],"macro":[["include_aseprite_sprite","Includes an Aseprite sprite sheet and Json export. For more information, see [`Sprite::load_aseprite_json`]. This macro will append “.png” and “.json” to the path provided and include both files in your binary."],["include_font","Embeds a font into your executable."],["include_texture","Embeds a texture in the binary."]],"mod":[["bundled_fonts","Feature-flag enabled fonts that are licensed under APL 2.0"]],"struct":[["Batch","A batch of shapes that can be rendered together."],["Color","A RGBA color with f32 components."],["DeviceId","Identifier of an input device."],["Fill","Shape fill options."],["Font","Font provides TrueType Font rendering"],["FrameRenderer","Renders frames created by a `Scene`."],["InputEvent","An Event from a device"],["Length","A one-dimensional distance, with value represented by `T` and unit of measurement `Unit`."],["Path","A geometric shape defined by a path."],["PathBuilder","Builds a [`Path`]."],["PreparedSpan","A formatted span of text that is ready to render. Cheap to clone."],["Raw","A unit representing physical pixels on a display."],["RedrawRequester","Allows requesting window refreshes outside of the event loop."],["RedrawStatus","Tracks when a window should be redrawn. Allows for rendering a frame immediately as well as scheduling a refresh in the future."],["Runtime","Runtime is designed to consume the main thread. For cross-platform compatibility, ensure that you call [`Runtime::run()`] from thee main thread."],["Scale","A scaling factor between two different units of measurement."],["Scaled","A unit representing Desktop publishing points/PostScript points. Measurements in this scale are equal to 1/72 of an imperial inch."],["Scene","The main rendering destination, usually interacted with through [`Target`]."],["Shape","A 2d shape."],["SingleWindowApplication","An [`Application`] implementation that begins with a single window."],["Sprite","A sprite is a renderable graphic with optional animations."],["SpriteAnimation","An animation of one or more [`SpriteFrame`]s."],["SpriteAnimations","A collection of [`SpriteAnimation`]s. This is an immutable object that shares data when cloned to minimize data copies."],["SpriteFrame","A single frame for a [`SpriteAnimation`]."],["SpriteMap","A collection of [`SpriteSource`]s."],["SpriteRotation","A rotation of a sprite."],["SpriteSheet","A collection of sprites from a single [`Texture`]."],["SpriteSource","A sprite’s source location and texture. Cheap to clone."],["SpriteSourceSublocation","A sub-location of a joined sprite."],["Stroke","A shape stroke (outline) options."],["Target","A render target"],["Texture","An image that can be used as a sprite. Cheap to clone."],["Unknown","A unit representing"],["WindowBuilder","A builder for a [`Window`]."]],"trait":[["Application","A trait that describes the application’s behavior."],["PointExt","Extension trait for [`Point`]."],["ShutdownCallback","A callback that can be invoked when a [`FrameRenderer`] is fully shut down."],["SizeExt","Extension trait for [`Size`]."],["SpriteCollection","A collection of sprites."],["Window","Trait to implement a Window"],["WindowCreator","Defines initial window properties."]],"type":[["Angle",""],["ControlPoint","A control point used to create curves."],["Endpoint","A point on a [`Path`]."],["Pixels","A measurement of length using [`Raw`] as the unit."],["Point","A type representing an x and y coordinate."],["Points","A measurement of length using [`Scaled`] as the unit."],["Rect","A type representing a [`Point`] and [`Size`]."],["ScanCode","Hardware-dependent keyboard scan code."],["ScreenScale","The scale used to convert between [`Points`] ([`Scaled`]) and [`Pixels`] ([`Raw`])."],["Size","A type representing a width and height."],["Vector","A type representing a vector with magnitudes x and y."]]});