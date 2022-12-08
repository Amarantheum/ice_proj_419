inlets = 1;
outlets = 7;

setinletassist(0, "bang input");
setoutletassist(0, "returns next chord in sequence");
setoutletassist(1, "returns next chord in sequence");
setoutletassist(2, "returns next chord in sequence");
setoutletassist(3, "returns next chord in sequence");
setoutletassist(4, "returns next chord in sequence");
setoutletassist(5, "returns count number")
setoutletassist(6, "return total chord number");

var count = 0;
var initialized = 0;

var chordArr = [
	[880, 0.5, 1., 1320, 0.5, 0.9, 1650, 0.9, 1.5, 2200, 1.8, 1.4, 1980, 0.5, 2.3],		// A
	[660, 0.5, 0.8, 990, 0.5, 1.2, 1110, 0.8, 1.3, 1480, 0.8, 1.4, 1160, 0.8, 1.3],		// E
	[990, 0.5, 0.6, 1480, 0.5, 1.3, 1320, 1.1, 0.9, 1860, 0.8, 1.1],					// B
	[740, 0.5, 0.7, 1110, 0.6, 0.9, 1865, 1.3, 0.9, 1240, 1.2, 0.8],					// F#
	[555, 0.5, 0.8, 830, 0.8, 1.2, 1040, 0.8, 1.3, 1400, 0.9, 1.5, 1570, 0.7, 1.2],		// C#
	[415, 0.5, 0.4, 587, 0.8, 0.9, 700, 0.7, 1.5, 990, 0.5, 1.5, 1245, 0.8, 1.3],		// G#
	[620, 0.5, 0.5, 830, 0.8, 0.8, 930, 1.3, 0.9, 1174, 0.9, 1.1, 1400, 0.8, 1.1],		// Eb
	[466, 0.5, 0.6, 700, 0.8, 1.1, 785, 1.1, 0.9, 1175, 0.8, 1.2],						// Bb
	[700, 0.5, 0.7, 930, 0.8, 1.2, 1110, 1.1, 0.9, 1480, 0.8, 1.1, 1980, 0.5, 1.3],		// F
	[1046, 0.5, 0.6, 1570, 0.8, 1.1, 2350, 0.5, 0.9, 1760, 0.8, 1.1, 1320, 0.7, 0.9],	// C
	[785, 0.5, 0.9, 880, 0.5, 1.0, 1174, 0.7, 0.9, 1400, 0.7, 1.2, 1318, 0.7, 0.8],		// G
	[1175, 0.6, 0.6, 1480, 0.7, 1.2, 1975, 0.8, 1.1, 2220, 0.8, 1.0, 3520, 0.4, 1.3],	// D
	[440, 0.5, 0.5, 660, 0.5, 0.5, 1650, 0.5, 1.5, 2200, 0.5, 1.4, 990, 0.5, 1.3],
	[660, 0.5, 0.5, 880, 0.5, 0.5, 1870, 0.5, 1.5, 2420, 0.5, 1.4, 1210, 0.5, 1.3],
	[990, 0.5, 0.6, 1400, 0.5, 1.3, 1300, 1.1, 0.9, 1800, 0.8, 1.1],
	[740, 0.5, 0.7, 1010, 0.6, 0.9, 1765, 1.3, 0.9, 1140, 1.2, 0.8],
	[555, 0.5, 0.8, 800, 0.8, 1.2, 940, 0.8, 1.3, 1200, 0.9, 1.5, 1270, 0.7, 1.2],
	[415, 0.5, 0.4, 567, 0.8, 0.9, 710, 0.7, 1.5, 950, 0.5, 1.5, 1145, 0.8, 1.3],
	[620, 0.5, 0.5, 800, 0.8, 0.8, 900, 1.3, 0.9, 1000, 0.9, 1.1, 1300, 0.8, 1.1],
	[466, 0.5, 0.6, 720, 0.8, 1.1, 975, 1.1, 0.9, 1175, 0.8, 1.2],	
	[700, 0.5, 0.7, 830, 0.8, 1.2, 1002, 1.1, 0.9, 1280, 0.8, 1.1, 1580, 0.5, 1.3],
	[1046, 0.5, 0.6, 1470, 0.8, 1.1, 2150, 0.5, 0.9, 1560, 0.8, 1.1, 1520, 0.7, 0.9],
	[785, 0.5, 0.9, 850, 0.5, 1.0, 1004, 0.7, 0.9, 1310, 0.7, 1.2, 1328, 0.7, 0.8],
	[1175, 0.6, 0.6, 1280, 0.7, 1.2, 1775, 0.8, 1.1, 2020, 0.8, 1.0, 3120, 0.4, 1.3],
	[440, 0.5, 0.5, 560, 0.5, 0.5, 1550, 0.5, 1.5, 2100, 0.5, 1.4, 1000, 0.5, 1.3],
	[660, 0.5, 0.5, 780, 0.5, 0.5, 870, 0.5, 1.5, 1220, 0.5, 1.4, 1210, 0.5, 1.3],
	[990, 0.5, 0.6, 1000, 0.5, 1.3, 1100, 1.1, 0.9, 1300, 0.8, 1.1],
	[740, 0.5, 0.7, 900, 0.6, 0.9, 1165, 1.3, 0.9, 1040, 1.2, 0.8],
	[555, 0.5, 0.8, 800, 0.8, 1.2, 940, 0.8, 1.3, 1000, 0.9, 1.5, 1270, 0.7, 1.2],
	[415, 0.5, 0.4, 567, 0.8, 0.9, 610, 0.7, 1.5, 750, 0.5, 1.5, 845, 0.8, 1.3],
	[620, 0.5, 0.5, 800, 0.8, 0.8, 1050, 1.3, 0.9, 1000, 0.9, 1.1, 1100, 0.8, 1.1],
	[466, 0.5, 0.6, 720, 0.8, 1.1, 775, 1.1, 0.9, 1075, 0.8, 1.2, 1085, 0.7, 1.1],	
	[700, 0.5, 0.7, 830, 0.8, 1.2, 1002, 1.1, 0.9, 1280, 0.8, 1.1, 1380, 0.5, 1.3],
	[1046, 0.5, 0.6, 1470, 0.8, 1.1, 1550, 0.5, 0.9, 1650, 0.8, 1.1, 1720, 0.7, 0.9],
	[785, 0.5, 0.9, 850, 0.5, 1.0, 950, 0.7, 0.9, 1310, 0.7, 1.2, 1328, 0.7, 0.8],
	[1175, 0.6, 0.6, 1280, 0.7, 1.2, 1775, 0.8, 1.1, 1920, 0.8, 1.0, 2120, 0.4, 1.3],
	[880, 0.5, 1., 1320, 0.5, 0.9, 1550, 0.9, 1.5, 2048, 1.8, 1.4, 1940, 0.5, 2.3],
	[660, 0.5, 0.8, 960, 0.5, 1.2, 987, 0.8, 1.3, 1279, 0.8, 1.4, 1048, 0.8, 1.3],
	[555, 0.5, 0.8, 802, 0.8, 1.2, 1100, 0.8, 1.3, 1340, 0.9, 1.5, 1500, 0.7, 1.2],	
	[740, 0.5, 0.7, 780, 0.6, 0.9, 800, 1.3, 0.9, 820, 1.2, 0.8],
	[466, 0.5, 0.6, 500, 0.8, 1.1, 785, 1.1, 0.9, 800, 0.8, 1.2, 900, 0.8, 1.1],
	[990, 0.5, 0.6, 1275, 0.5, 1.3, 1188, 1.1, 0.9, 1751, 0.8, 1.1],
	[740, 0.5, 0.7, 1046, 0.6, 0.9, 1382, 1.3, 0.9, 1760, 1.2, 0.8],
	[555, 0.5, 0.8, 783, 0.8, 1.2, 830, 0.8, 1.3, 1380, 0.9, 1.5, 1452, 0.7, 1.2],
	[440, 0.5, 0.5, 523, 0.5, 0.5, 554, 0.5, 1.5, 1244, 0.5, 1.4, 622, 0.5, 1.3],
	[660, 0.5, 0.5, 680, 0.5, 0.5, 670, 0.5, 1.5, 2420, 0.5, 1.4, 1210, 0.5, 1.3],
	[990, 0.5, 0.6, 1480, 0.5, 1.3, 995, 1.1, 0.9, 1000, 0.8, 1.1, 1510, 0.7, 1.2],
	[454, 0.5, 0.6, 455, 0.8, 1.1, 456, 1.1, 0.9, 1212, 0.8, 1.2, 1215, 0.7, 1.1, 1216, 0.7, 1.1, 1217, 0.7, 1.1],
	[460, 0.5, 0.6, 459, 0.8, 1.1, 458, 1.1, 0.9, 1092, 0.8, 1.2, 1095, 0.7, 1.1, 1096, 0.7, 1.1, 1097, 0.7, 1.1],
	[466, 0.5, 0.6, 465, 0.8, 1.1, 467, 1.1, 0.9, 468, 0.8, 1.2, 1085, 0.7, 1.1, 1086, 0.7, 1.1, 1087, 0.7, 1.1],
	[474, 0.5, 0.6, 485, 0.8, 1.1, 486, 1.1, 0.9, 484, 0.8, 1.2, 1520, 0.7, 1.3, 1522, 0.7, 1.3, 1524, 0.7, 1.4],
	[480, 0.5, 0.6, 485, 0.8, 1.1, 484, 1.1, 0.9, 494, 0.8, 1.2, 1720, 0.7, 1.5, 1722, 0.7, 1.5, 1724, 0.7, 1.5],
	[484, 0.5, 0.6, 495, 0.8, 1.1, 496, 1.1, 0.9, 494, 0.8, 1.2, 1820, 0.7, 1.1, 1822, 0.7, 1.1, 1824, 0.7, 1.1],
	[488, 0.5, 0.6, 498, 0.8, 1.1, 497, 1.1, 0.9, 500, 0.8, 1.2, 1920, 0.7, 1.1, 1922, 0.7, 1.1, 1924, 0.7, 1.1],
	[1760, 0.3, 0.2],
];

function bang() {
	if (!initialized) { return; }
	// stop if at last chord
	if (count > chordArr.length) {
		outlet(count % 5, [0, 0, 0]);
		count++;
		return;
	}
	// iterate through outlets as resonators can only play one chord
	// at any time
	outlet(count % 5, chordArr[count]);
	count++;
	// uncomment to loop chords for testing
	if (count > chordArr.length) {
		count = 45;
	}
	outlet(5, count);
}

function init() {
	// output number of chords in array
	initialized = 1;
	outlet(6, chordArr.length);
}

function reset() {
	// reset chords
	count = 0;
	outlet(5, count);
}