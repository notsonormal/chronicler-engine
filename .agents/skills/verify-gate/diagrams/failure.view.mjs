// The two failures that the dashboard of one Game shows: the failure line of the status
// display, and the Options failure of the dock.
export const title = "The failure line and the Options failure of one Game";

const line = (v) => (v.failLine[1] === 0 ? "No failure line" : "A failure line");
const options = (v) => (v.optFail[1] === 0 ? "no Options failure" : "an Options failure");

export const state = (v) => `${line(v)}, ${options(v)}`;

export const group = (v) => (v.failLine[1] === 0 && v.optFail[1] === 0 ? "Clear" : "Failure");
export const colors = { Clear: "teal", Failure: "orange" };

export const step = (action, from, to) => {
  switch (action) {
    case "YouStartAJob": return "the accept of a request";
    case "TheAcceptCannotRead": return "an accept cannot read the story";
    case "YouPressStop": return "the player presses Stop";
    case "YouSwitchOrDeleteASwipe": return "a Swipe switch or delete";
    case "TheTextIsSaved": return "the save of a Narration";
    case "TheOptionsAreSaved":
      return to.optFail[1] === 0 ? "the save of a set of Options" : "the save of the Options failure";
    case "TheFailureLineIsSaved": return "the save of the failure line of a Job";
    case "TheRunAccepts":
      return to.failLine[1] === 0 ? "the accept of a request" : "an accept cannot read the story";
  }
  return null;
};
