inlets = 1;
outlets = 6;

setinletassist(0, "bang input");
setoutletassist(0, "returns next chord in sequence");
setoutletassist(1, "returns next chord in sequence");
setoutletassist(2, "returns next chord in sequence");
setoutletassist(3, "returns next chord in sequence");
setoutletassist(4, "returns next chord in sequence");
setoutletassist(5, "returns count number")

var count = 0;
var chordArr = [
	[880, 0.5, 1., 1320, 0.5, 0.9, 1650, 0.9, 1.5, 2200, 1.8, 1.4, 1980, 0.5, 2.3],
	[660, 0.5, 0.8, 990, 0.5, 1.2, 1110, 0.8, 1.3, 1480, 0.8, 1.4, 1160, 0.8, 1.3],
	[990, 0.5, 0.6, 1480, 0.5, 1.3, 1320, 1.1, 0.9, 1860, 0.8, 1.1],
	[740, 0.5, 0.7, 1110, 0.6, 0.9, 1661, 1.3, 0.9, 1244, 1.2, 0.8],
	[440, 0.5, 0.5, 660, 0.5, 0.5, 1650, 0.5, 1.5, 2200, 0.5, 1.4, 990, 0.5, 1.3],
	[660, 0.5, 0.5, 880, 0.5, 0.5, 1870, 0.5, 1.5, 2420, 0.5, 1.4, 1210, 0.5, 1.3]
];

function bang() {
	// iterate through outlets as resonators can only play one chord
	// at any time
	outlet(count % 5, chordArr[count]);
	count++;
	// just keeping this here for testing so i don't have to reset
	// the patch constantly
	if (count > chordArr.length) {
		count = 0;
	}
	outlet(5, count);
}