import sys

from dotenv import load_dotenv

from .cli import app

# Load environment variables from .env file
load_dotenv()


def main():
    try:
        app(standalone_mode=True)
    except SystemExit as e:
        sys.exit(e.code)


if __name__ == "__main__":
    main()

# Make main() the callable when importing __main__
__call__ = main
