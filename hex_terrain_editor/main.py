"""Application entry point for the Hex Terrain Map Editor."""

from .app import HexTerrainEditorApp


def main() -> None:
    app = HexTerrainEditorApp()
    app.run()


if __name__ == "__main__":
    main()
