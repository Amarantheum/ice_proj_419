inlets = 1;
outlets = 1;

setinletassist(0, "bang input");
setoutletassist(0, "returns next chord in sequence");

var count = 0;
var chordArr = [[880, 0.5, 1., 1320, 0.5, 0.9, 1650, 0.9, 1.5, 2200, 1.8, 1.4,
1980, 0.5, 2.3], [440, 0.5, 0.5, 660, 0.5, 0.5, 1650, 0.5, 1.5, 2200, 0.5,
1.4, 990, 0.5, 1.3], [660, 0.5, 0.5, 880, 0.5, 0.5, 1870, 0.5, 1.5, 2420, 0.5,
1.4, 1210, 0.5, 1.3]];

function bang() {
	outlet(0, chordArr[count]);
	count++;
	// just keeping this here for testing so i don't have to reset
	// the patch constantly
	if (count > chordArr.length) {
		count = 0;
	}
}