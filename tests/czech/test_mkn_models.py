"""Unit tests for MKN-10 Pydantic models."""

from biomcp.czech.mkn.models import (
    Diagnosis,
    DiagnosisCategory,
    DiagnosisHierarchy,
    Modifier,
)


class TestMknModels:
    def test_diagnosis_hierarchy(self):
        h = DiagnosisHierarchy(
            chapter="X",
            chapter_name="Nemoci dychaci soustavy",
            block="J00-J06",
            block_name="Akutni infekce",
            category="J06",
        )
        assert h.chapter == "X"

    def test_modifier(self):
        m = Modifier(code="M01", name="Modifier 1")
        assert m.code == "M01"

    def test_diagnosis_minimal(self):
        d = Diagnosis(code="J06.9", name_cs="Test")
        assert d.source == "UZIS/MKN-10"
        assert d.includes == []
        assert d.excludes == []
        assert d.modifiers == []

    def test_diagnosis_full(self):
        d = Diagnosis(
            code="J06.9",
            name_cs="Akutni infekce NS",
            name_en="Acute upper respiratory infection",
            definition="Definition text",
            hierarchy=DiagnosisHierarchy(
                chapter="X",
                chapter_name="Ch X",
                block="J00-J06",
                block_name="Block",
                category="J06",
            ),
            includes=["rhinitis"],
            excludes=["pneumonia"],
            modifiers=[
                Modifier(code="M1", name="Mod1")
            ],
        )
        assert d.hierarchy.chapter == "X"
        assert len(d.includes) == 1

    def test_diagnosis_category(self):
        c = DiagnosisCategory(
            code="X",
            name="Chapter X",
            type="chapter",
            children=[
                DiagnosisCategory(
                    code="J00-J06",
                    name="Block",
                    type="block",
                    parent_code="X",
                )
            ],
        )
        assert len(c.children) == 1
        assert c.children[0].parent_code == "X"

    def test_diagnosis_category_no_children(self):
        c = DiagnosisCategory(
            code="J06.9",
            name="Leaf",
            type="category",
        )
        assert c.children == []
        assert c.parent_code is None
