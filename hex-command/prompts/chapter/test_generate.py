"""The chapter entry point must compose story progression and event groups."""
import unittest

import generate


class ChapterCompositionTests(unittest.TestCase):
    def test_story_transitions_and_correlated_events_share_one_output(self):
        opening_kind = next(iter(generate.BATTLE_TRANSITIONS["opening-middle"]))
        middle_kind = next(iter(generate.BATTLE_TRANSITIONS["middle-middle"]))
        cases = [
            ("opening", None, None),
            ("middle", "opening", opening_kind),
            ("finale", "middle", middle_kind),
        ]
        for position, previous_position, previous_kind in cases:
            with self.subTest(position=position):
                result = generate.sample(3, 42, position, previous_position, previous_kind)
                allowed = generate.story_weights(position, previous_position, previous_kind)
                for chapter in result["chapters"]:
                    self.assertIn(chapter["story_kind"], allowed)
                    self.assertTrue(chapter["story_kind_definition"])
                    self.assertTrue(chapter["objective_flow"])
                    self.assertTrue(chapter["outcomes"])
                    self.assertTrue(chapter["linked_events"])
                    self.assertIsInstance(chapter["independent_groups"], list)
                    self.assertIsInstance(chapter["optional_encounters"], list)
                    self.assertEqual("ending_kind" in chapter, position == "finale")
                rendered = generate.render_chapters(result)
                self.assertIn("**Story kind:**", rendered)
                self.assertIn("**Linked events:**", rendered)
                self.assertIn("**Independent developments (unordered):**", rendered)
                self.assertIn("**Optional encounters:**", rendered)


if __name__ == "__main__":
    unittest.main()
