"""Ramer-Douglas-Peucker simplification and Bézier curve fitting for path features."""

from __future__ import annotations

import math


def perpendicular_distance(
    point: tuple[float, float],
    line_start: tuple[float, float],
    line_end: tuple[float, float],
) -> float:
    """Compute perpendicular distance from a point to a line segment."""
    dx = line_end[0] - line_start[0]
    dy = line_end[1] - line_start[1]
    length_sq = dx * dx + dy * dy

    if length_sq == 0:
        return math.hypot(point[0] - line_start[0], point[1] - line_start[1])

    t = max(0, min(1, (
        (point[0] - line_start[0]) * dx + (point[1] - line_start[1]) * dy
    ) / length_sq))

    proj_x = line_start[0] + t * dx
    proj_y = line_start[1] + t * dy

    return math.hypot(point[0] - proj_x, point[1] - proj_y)


def rdp_simplify(
    points: list[tuple[float, float]], epsilon: float
) -> list[tuple[float, float]]:
    """Ramer-Douglas-Peucker polyline simplification.

    Args:
        points: Input polyline as list of (x, y) tuples.
        epsilon: Maximum allowed perpendicular distance from simplified line.

    Returns:
        Simplified polyline.
    """
    if len(points) <= 2:
        return list(points)

    dmax = 0.0
    index = 0
    for i in range(1, len(points) - 1):
        d = perpendicular_distance(points[i], points[0], points[-1])
        if d > dmax:
            dmax = d
            index = i

    if dmax > epsilon:
        left = rdp_simplify(points[: index + 1], epsilon)
        right = rdp_simplify(points[index:], epsilon)
        return left[:-1] + right
    else:
        return [points[0], points[-1]]


def fit_bezier_to_polyline(
    points: list[tuple[float, float]],
) -> list[tuple[tuple[float, float], tuple[float, float], tuple[float, float], tuple[float, float]]]:
    """Convert simplified polyline into cubic Bézier curves.

    Uses Catmull-Rom to Bézier conversion for smooth tangents.

    Returns:
        List of (p0, p1, p2, p3) cubic Bézier control point tuples.
    """
    if len(points) < 2:
        return []

    curves = []
    for i in range(len(points) - 1):
        p0 = points[i]
        p3 = points[i + 1]

        prev = points[i - 1] if i > 0 else p0
        next_pt = points[i + 2] if i < len(points) - 2 else p3

        # Catmull-Rom tangents
        t0 = ((p3[0] - prev[0]) / 6.0, (p3[1] - prev[1]) / 6.0)
        t1 = ((next_pt[0] - p0[0]) / 6.0, (next_pt[1] - p0[1]) / 6.0)

        p1 = (p0[0] + t0[0], p0[1] + t0[1])
        p2 = (p3[0] - t1[0], p3[1] - t1[1])

        curves.append((p0, p1, p2, p3))

    return curves


def evaluate_bezier(
    p0: tuple[float, float],
    p1: tuple[float, float],
    p2: tuple[float, float],
    p3: tuple[float, float],
    t: float,
) -> tuple[float, float]:
    """Evaluate a cubic Bézier curve at parameter t in [0, 1]."""
    u = 1.0 - t
    uu = u * u
    uuu = uu * u
    tt = t * t
    ttt = tt * t

    x = uuu * p0[0] + 3 * uu * t * p1[0] + 3 * u * tt * p2[0] + ttt * p3[0]
    y = uuu * p0[1] + 3 * uu * t * p1[1] + 3 * u * tt * p2[1] + ttt * p3[1]
    return (x, y)


def bezier_to_polyline(
    p0: tuple[float, float],
    p1: tuple[float, float],
    p2: tuple[float, float],
    p3: tuple[float, float],
    segments: int = 20,
) -> list[tuple[float, float]]:
    """Convert a cubic Bézier curve to a polyline approximation."""
    return [evaluate_bezier(p0, p1, p2, p3, t / segments) for t in range(segments + 1)]
