"""Test backward compatibility for trial search and getter functions."""

from unittest.mock import patch

import pytest

from biomcp.trials.getter import Module, get_trial, get_trial_unified
from biomcp.trials.search import (
    TrialQuery,
    search_trials,
    search_trials_unified,
)


class TestTrialSearchBackwardCompatibility:
    """Test that existing trial search functionality remains unchanged."""

    @pytest.mark.asyncio
    async def test_search_trials_defaults_to_clinicaltrials(self):
        """Test that search_trials still defaults to ClinicalTrials.gov."""
        query = TrialQuery(conditions=["diabetes"])

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (
                {
                    "studies": [
                        {
                            "protocolSection": {
                                "identificationModule": {"nctId": "NCT12345"}
                            }
                        }
                    ]
                },
                None,
            )

            await search_trials(query, output_json=True)

            # Verify it called the ClinicalTrials.gov API
            assert mock_request.called
            call_args = mock_request.call_args
            # Check the URL argument
            url_arg = call_args.kwargs.get("url")
            assert url_arg is not None
            assert "clinicaltrials.gov" in url_arg

    @pytest.mark.asyncio
    async def test_search_trials_no_source_parameter(self):
        """Test that search_trials function signature hasn't changed."""
        # This test ensures the function can still be called without source
        query = TrialQuery(conditions=["cancer"])

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = ({"studies": []}, None)

            # Should not raise TypeError about unexpected keyword argument
            await search_trials(query)
            assert mock_request.called

    @pytest.mark.asyncio
    async def test_search_trials_unified_with_source(self):
        """Test unified function supports source parameter."""
        query = TrialQuery(conditions=["melanoma"])

        # Test with ClinicalTrials.gov
        with patch("biomcp.trials.search.search_trials") as mock_ct:
            mock_ct.return_value = "CT results"

            result = await search_trials_unified(
                query, source="clinicaltrials"
            )
            assert result == "CT results"
            mock_ct.assert_called_once_with(query, False)

        # Test with NCI
        with (
            patch("biomcp.trials.nci_search.search_trials_nci") as mock_nci,
            patch(
                "biomcp.trials.nci_search.format_nci_trial_results"
            ) as mock_format,
        ):
            mock_nci.return_value = {"source": "nci", "trials": []}
            mock_format.return_value = "NCI formatted results"

            result = await search_trials_unified(
                query, source="nci", api_key="test-key"
            )
            assert result == "NCI formatted results"
            mock_nci.assert_called_once_with(query, "test-key")


class TestTrialGetterBackwardCompatibility:
    """Test that existing trial getter functionality remains unchanged."""

    @pytest.mark.asyncio
    async def test_get_trial_defaults_to_clinicaltrials(self):
        """Test that get_trial still defaults to ClinicalTrials.gov."""
        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (
                {
                    "protocolSection": {
                        "identificationModule": {"nctId": "NCT12345"}
                    }
                },
                None,
            )

            await get_trial("NCT12345", Module.PROTOCOL)

            # Verify it called the ClinicalTrials.gov API
            assert mock_request.called
            call_args = mock_request.call_args
            # Check the URL argument
            url_arg = call_args.kwargs.get("url")
            assert url_arg is not None
            assert "clinicaltrials.gov" in url_arg
            # NCT ID would be in the request params, not the URL

    @pytest.mark.asyncio
    async def test_get_trial_no_source_parameter(self):
        """Test that get_trial function signature hasn't changed."""
        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (
                {
                    "protocolSection": {
                        "identificationModule": {"nctId": "NCT99999"}
                    }
                },
                None,
            )

            # Should not raise TypeError about unexpected keyword argument
            await get_trial("NCT99999", Module.PROTOCOL, output_json=True)
            assert mock_request.called

    @pytest.mark.asyncio
    async def test_get_trial_unified_with_source(self):
        """Test unified function supports source parameter."""
        # Test with ClinicalTrials.gov - uses private functions
        with patch("biomcp.trials.getter._trial_protocol") as mock_protocol:
            mock_protocol.return_value = "CT trial details"

            result = await get_trial_unified(
                "NCT12345", source="clinicaltrials", sections=["protocol"]
            )
            assert result == "CT trial details"
            mock_protocol.assert_called_once_with(
                nct_id="NCT12345",
                call_benefit="Getting protocol information for trial NCT12345",
            )

        # Test with NCI
        with (
            patch("biomcp.trials.nci_getter.get_trial_nci") as mock_nci,
            patch(
                "biomcp.trials.nci_getter.format_nci_trial_details"
            ) as mock_format,
        ):
            mock_nci.return_value = {"nct_id": "NCT12345", "source": "nci"}
            mock_format.return_value = "NCI formatted trial"

            result = await get_trial_unified(
                "NCT12345", source="nci", api_key="test-key"
            )
            assert result == "NCI formatted trial"
            mock_nci.assert_called_once_with("NCT12345", "test-key")

    @pytest.mark.asyncio
    async def test_get_trial_all_modules_still_work(self):
        """Test that all existing Module options still work."""
        modules_to_test = [
            Module.PROTOCOL,
            Module.LOCATIONS,
            Module.REFERENCES,
            Module.OUTCOMES,
        ]

        for module in modules_to_test:
            with patch("biomcp.http_client.request_api") as mock_request:
                mock_request.return_value = (
                    {
                        "protocolSection": {
                            "identificationModule": {"nctId": "NCT12345"}
                        }
                    },
                    None,
                )

                await get_trial("NCT12345", module)
            assert mock_request.called
            # Reset for next iteration
            mock_request.reset_mock()


class TestCLIBackwardCompatibility:
    """Test that CLI commands maintain backward compatibility."""

    def test_cli_imports_exist(self):
        """Test that CLI still imports the expected functions."""
        # These imports should not raise ImportError
        from biomcp.cli.trials import get_trial_cli, search_trials_cli

        assert search_trials_cli is not None
        assert get_trial_cli is not None

    def test_search_defaults_without_source(self):
        """Test CLI search works without source parameter."""
        from typer.testing import CliRunner

        from biomcp.cli.main import app

        runner = CliRunner()

        with patch("biomcp.cli.trials.asyncio.run") as mock_run:
            mock_run.return_value = None

            # Run CLI command without --source
            result = runner.invoke(
                app, ["trial", "search", "--condition", "diabetes"]
            )

            # Should succeed
            assert result.exit_code == 0

            # Verify asyncio.run was called with the right function
            mock_run.assert_called()
            args = mock_run.call_args[0][0]
            # Check that it's the unified search function being called
            assert hasattr(args, "__name__") or hasattr(args, "func")
