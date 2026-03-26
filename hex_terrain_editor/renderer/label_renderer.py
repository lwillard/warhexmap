"""City/feature name label rendering."""

from __future__ import annotations

from PIL import ImageDraw, ImageFont


def render_label(
    draw: ImageDraw.ImageDraw,
    text: str,
    x: float,
    y: float,
    font_size: int = 12,
    color: tuple[int, int, int] = (30, 30, 30),
    halo_color: tuple[int, int, int, int] = (255, 255, 255, 180),
) -> None:
    """Render a text label with a semi-transparent halo for readability."""
    try:
        font = ImageFont.truetype("arial.ttf", font_size)
    except (OSError, IOError):
        font = ImageFont.load_default()

    bbox = draw.textbbox((x, y), text, font=font, anchor="mm")

    # Draw halo (slightly expanded background)
    pad = 2
    draw.rectangle(
        [bbox[0] - pad, bbox[1] - pad, bbox[2] + pad, bbox[3] + pad],
        fill=halo_color,
    )

    draw.text((x, y), text, fill=color, font=font, anchor="mm")
