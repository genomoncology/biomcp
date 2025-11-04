"""Tests for prefetch disable functionality."""

import os
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.constants import PREFETCH_DISABLED_ENV_VAR


class TestPrefetchEnvironmentVariable:
    """Test prefetch behavior with environment variable."""

    @pytest.mark.asyncio
    async def test_prefetch_disabled_true(self):
        """Test that prefetch is skipped when env var is 'true'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "true"}),
            patch("biomcp.prefetch.start_prefetching") as mock_prefetch,
        ):
            # Import core module after setting env var
            from biomcp.core import lifespan

            # Execute lifespan
            async with lifespan(None):
                pass

            # Prefetch should NOT be called
            mock_prefetch.assert_not_called()

    @pytest.mark.asyncio
    async def test_prefetch_disabled_1(self):
        """Test that prefetch is skipped when env var is '1'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "1"}),
            patch("biomcp.prefetch.start_prefetching") as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_not_called()

    @pytest.mark.asyncio
    async def test_prefetch_disabled_yes(self):
        """Test that prefetch is skipped when env var is 'yes'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "yes"}),
            patch("biomcp.prefetch.start_prefetching") as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_not_called()

    @pytest.mark.asyncio
    async def test_prefetch_disabled_case_insensitive(self):
        """Test that env var is case-insensitive."""
        test_values = ["TRUE", "True", "YES", "Yes", "1"]

        for value in test_values:
            with (
                patch.dict(
                    os.environ, {PREFETCH_DISABLED_ENV_VAR: value}, clear=True
                ),
                patch("biomcp.prefetch.start_prefetching") as mock_prefetch,
            ):
                from biomcp.core import lifespan

                async with lifespan(None):
                    pass

                mock_prefetch.assert_not_called()

    @pytest.mark.asyncio
    async def test_prefetch_enabled_false(self):
        """Test that prefetch runs when env var is 'false'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "false"}),
            patch(
                "biomcp.prefetch.start_prefetching", new_callable=AsyncMock
            ) as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            # Prefetch SHOULD be called
            mock_prefetch.assert_called_once()

    @pytest.mark.asyncio
    async def test_prefetch_enabled_0(self):
        """Test that prefetch runs when env var is '0'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "0"}),
            patch(
                "biomcp.prefetch.start_prefetching", new_callable=AsyncMock
            ) as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_called_once()

    @pytest.mark.asyncio
    async def test_prefetch_enabled_no(self):
        """Test that prefetch runs when env var is 'no'."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "no"}),
            patch(
                "biomcp.prefetch.start_prefetching", new_callable=AsyncMock
            ) as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_called_once()

    @pytest.mark.asyncio
    async def test_prefetch_enabled_empty_string(self):
        """Test that prefetch runs when env var is empty string."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: ""}),
            patch(
                "biomcp.prefetch.start_prefetching", new_callable=AsyncMock
            ) as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_called_once()

    @pytest.mark.asyncio
    async def test_prefetch_enabled_not_set(self):
        """Test that prefetch runs when env var is not set (default)."""
        # Ensure env var is not set
        env = os.environ.copy()
        env.pop(PREFETCH_DISABLED_ENV_VAR, None)

        with (
            patch.dict(os.environ, env, clear=True),
            patch(
                "biomcp.prefetch.start_prefetching", new_callable=AsyncMock
            ) as mock_prefetch,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            mock_prefetch.assert_called_once()

    @pytest.mark.asyncio
    async def test_prefetch_failure_does_not_prevent_startup(self):
        """Test that prefetch failures don't prevent server startup."""
        with (
            patch.dict(os.environ, {}, clear=True),
            patch(
                "biomcp.prefetch.start_prefetching",
                new_callable=AsyncMock,
                side_effect=Exception("Prefetch failed"),
            ),
        ):
            from biomcp.core import lifespan

            # Should not raise exception
            async with lifespan(None):
                pass

    @pytest.mark.asyncio
    async def test_prefetch_disabled_logs_info_message(self):
        """Test that disabling prefetch logs an info message."""
        with (
            patch.dict(os.environ, {PREFETCH_DISABLED_ENV_VAR: "true"}),
            patch("biomcp.core.logger") as mock_logger,
        ):
            from biomcp.core import lifespan

            async with lifespan(None):
                pass

            # Check that info was logged
            mock_logger.info.assert_called_once_with(
                "Prefetching disabled via environment variable"
            )


class TestCLIFlag:
    """Test CLI flag behavior."""

    def test_disable_prefetch_flag_accepted(self):
        """Test that --disable-prefetch flag is accepted by the CLI."""
        from typer.testing import CliRunner

        from biomcp.cli.main import app

        runner = CliRunner(mix_stderr=False)

        # Mock the actual server running
        with patch("biomcp.cli.server.run_stdio_server"):
            # Run with --disable-prefetch flag
            result = runner.invoke(app, ["run", "--disable-prefetch"])

            # Should execute without errors (exit code 0)
            assert result.exit_code == 0

    def test_without_disable_prefetch_flag(self):
        """Test that server runs without --disable-prefetch flag."""
        from typer.testing import CliRunner

        from biomcp.cli.main import app

        runner = CliRunner(mix_stderr=False)

        # Mock the actual server running
        with patch("biomcp.cli.server.run_stdio_server"):
            # Run without --disable-prefetch flag
            result = runner.invoke(app, ["run"])

            # Should execute without errors
            assert result.exit_code == 0

    def test_disable_prefetch_flag_with_worker_mode(self):
        """Test that --disable-prefetch works with worker mode."""
        from typer.testing import CliRunner

        from biomcp.cli.main import app

        runner = CliRunner(mix_stderr=False)

        with patch("biomcp.cli.server.run_http_server"):
            result = runner.invoke(
                app,
                ["run", "--mode", "worker", "--disable-prefetch"],
            )

            assert result.exit_code == 0

    def test_disable_prefetch_flag_with_streamable_http_mode(self):
        """Test that --disable-prefetch works with streamable_http mode."""
        from typer.testing import CliRunner

        from biomcp.cli.main import app

        runner = CliRunner(mix_stderr=False)

        with patch("biomcp.cli.server.run_http_server"):
            result = runner.invoke(
                app,
                [
                    "run",
                    "--mode",
                    "streamable_http",
                    "--disable-prefetch",
                ],
            )

            assert result.exit_code == 0

    def test_run_server_function_sets_env_var(self):
        """Test that run_server function sets environment variable when flag is True."""
        from biomcp.cli.server import run_server

        # Clear env var first
        original_value = os.environ.get(PREFETCH_DISABLED_ENV_VAR)
        if PREFETCH_DISABLED_ENV_VAR in os.environ:
            del os.environ[PREFETCH_DISABLED_ENV_VAR]

        try:
            with patch("biomcp.cli.server.run_stdio_server"):
                # Call run_server with disable_prefetch=True
                run_server(disable_prefetch=True)

                # Check that env var was set
                assert os.environ.get(PREFETCH_DISABLED_ENV_VAR) == "true"
        finally:
            # Restore original value
            if original_value is not None:
                os.environ[PREFETCH_DISABLED_ENV_VAR] = original_value
            elif PREFETCH_DISABLED_ENV_VAR in os.environ:
                del os.environ[PREFETCH_DISABLED_ENV_VAR]

    def test_run_server_function_does_not_set_env_var_when_false(self):
        """Test that run_server function doesn't set env var when flag is False."""
        from biomcp.cli.server import run_server

        # Clear env var first
        original_value = os.environ.get(PREFETCH_DISABLED_ENV_VAR)
        if PREFETCH_DISABLED_ENV_VAR in os.environ:
            del os.environ[PREFETCH_DISABLED_ENV_VAR]

        try:
            with patch("biomcp.cli.server.run_stdio_server"):
                # Call run_server with disable_prefetch=False (default)
                run_server(disable_prefetch=False)

                # Check that env var was NOT set
                assert os.environ.get(PREFETCH_DISABLED_ENV_VAR) != "true"
        finally:
            # Restore original value
            if original_value is not None:
                os.environ[PREFETCH_DISABLED_ENV_VAR] = original_value
            elif PREFETCH_DISABLED_ENV_VAR in os.environ:
                del os.environ[PREFETCH_DISABLED_ENV_VAR]
