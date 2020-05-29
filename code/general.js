// ins patch (post-fx), set an index, delete count, then elements to insert. Does not store offsets, so it's quite volatile. Or maybe it shows you the patch steps and you select the appropriate index.
// continue(index) as the user response
// entering null or undefined skips that index
// So it's not inserting or deleting, it's both. It's not prepending or appending, it could be either or both. It's...
// Patching Part II? Map0010.patch.json, in the "splice" node.
// "splice": [{"path":[1,5],"":""}]
// Actually just do the below, then download a patched JSON instead of a patched binary. Then you splice the JSON later. "splice": true

// Maybe add appending and prepending? Execute those instructions before anything else, then have a common offset in addition.
// a = events.length
// per_event_offset + a --- if it's the same event
// splice(index, a, ...events)
// Actually, never mind. If I need to add events, I'll manually do it after a patch is applied and before that patched data gets compiled.
// The event will be noted and it won't need to happen too often anyways...

// Space out based on branches, ie if the indent is different or code === 10, ---
// Potential "choices" node (and potentially mark it so it appears in the text as well?):
/*
"choices":
[
	{
		"path": [142,2,15,16,18],
		"choices":
		[
			{
				"original": "choice1 de",
				"patch": "choice1 en"
			},
			{
				"original": "choice2 de",
				"patch": "choice2 en"
			}
		]
	}
]
*/

let chars;
let stack = {};
let settings = {
	disableDownloading: false,
	hasOther: false,
	immediateDownloads: false,
	showReaderLog: false,
	delay: 300,
	legacyDialogue: false,
	retainOriginalLineEndings: false,
	hasPortraitCondition(entry) {return true;},
	downloadDialoguelessMaps: false,
	downloadBatchedText: true,
	manualPortraits: false,
	deleteFalseInPortraits: true,
	downloadAsZip: true,
	suspendErrors: false
};
let text = ''; // copy(text) to copy the text-form dialogue.
let scheduler = new Scheduler();
let filter = [10140,20140,10610]; // Event commands to extract when not extracting all strings.
let zip = new JSZip();

// Removes all escape characters and replaces \\n[#] with characters if "chars.json" is provided.
function handleSpecials(line)
{
	line = line.replace(/\\[^nv](\[\d*\])*/g, '');
	
	for(let c in chars)
		line = line.replace(`\\n[${c}]`, chars[c]);
	
	return line;
}

// Squishes an array of lines into one.
function compressLines(lines, overrideRetain = false)
{
	let s = '';
	
	for(let i = 0; i < lines.length; i++)
	{
		let line = lines[i];
		
		if(i !== lines.length-1)
		{
			if(settings.retainOriginalLineEndings || overrideRetain)
				s += line + '\n';
			else
				s += line.trimStart().trimEnd() + ' ';
		}
		else
			s += line;
	}
	
	return s;
}

// Common Events Dialogue: path = [event #, page #, pos, length]
function generatePatchMap(data)
{
	let dialogue = [];
	let other = [];
	let e = data[81];
	
	for(let ev in e)
	{
		let ev_num = parseInt(ev);
		let p = e[ev][5];
		
		for(let pg in p)
		{
			let pg_num = parseInt(pg);
			generatePatchEvent(dialogue, p[pg][52], [ev_num, pg_num], other);
		}
	}
	
	let out = {};
	
	if(dialogue.length > 0)
		out.dialogue = dialogue;
	
	if(other.length > 0)
		out.other = other;
	
	return out;
}

// Vocabulary: path = [global_id, term #]
// Common Events Dialogue: path = [event #, pos, length]
function generatePatchDatabase(data)
{
	let dialogue = [];
	let other = [];
	let e = data[25];
	
	for(let ev in e)
	{
		let ev_num = parseInt(ev);
		generatePatchEvent(dialogue, e[ev][22], [ev_num], other);
	}
	
	for(let choice of other)
		choice.path = [25, choice.path[0], 22, choice.path[1], 2];
	
	e = data[21];
	
	for(let id in e)
	{
		id = parseInt(id);
		other.push({
			path: [21, id],
			original: e[id],
			patch: e[id]
		});
	}
	
	return {dialogue: dialogue, other: other};
}

// My goal with the new patch format is to avoid duplicating text.
// So have an extra node for the original text and then the patched lines below,
// rather than having a separate text file which you'll then have to update both.
// Also, the if all indents of a node are the same, it'll be just one number.
function generatePatchEvent(patch, cmd, path, other)
{
	// 10110 as A
	// 20110 as B
	// A o --> [A]
	// A A --> [A] [A]
	// A B --> [A,B]
	// A B B --> [A,B,B]
	// A B A --> [A,B] [A]
	for(let i = 0, len = cmd.length, last = null, lines = [], indent = [], params = [], hasIndent = false, hasParams = false, start = 0, mainIndent = 0, portrait = true; i < len; i++)
	{
		let code = cmd[i][0];
		let ind = cmd[i][1];
		let str = cmd[i][2];
		let prm = cmd[i][3];
		
		if((last === 10110 && code !== 20110) || (last === 20110 && code !== 20110))
		{
			let entry = {};
			entry.path = [...path, start, lines.length];
			if(hasIndent)
				entry.indent = mainIndent !== -1 ? mainIndent : indent;
			if(hasParams)
				entry.parameters = params;
			entry.portrait = portrait; // I want the order to be a certain way, so I'm declaring it true here then changing it afterwards if need be.
			entry.original = handleSpecials(compressLines(lines));
			if(settings.legacyDialogue)
				entry.lines = lines;
			else
				entry.patch = compressLines(lines);
			if(settings.manualPortraits)
				entry.portrait = settings.hasPortraitCondition(entry);
			if((settings.deleteFalseInPortraits && !entry.portrait) || settings.legacyDialogue) // But if you're generating a legacy patch, don't include it because it wasn't part of that format.
				delete entry.portrait;
			patch.push(entry);
			lines = [];
			indent = [];
			params = [];
			hasIndent = false;
			hasParams = false;
		}
		
		if(code === 10110)
		{
			if(lines.length > 0)
				console.warn(lines);
			if(indent.length > 0)
				console.warn(indent);
			if(params.length > 0)
				console.warn(params);
			start = i;
			lines = [str];
			indent = [ind];
			params = [prm];
			mainIndent = ind;
			if(ind != 0)
				hasIndent = true;
			if(prm.length != 0)
				hasParams = true;
		}
		else if(code === 20110)
		{
			lines.push(str);
			indent.push(ind);
			params.push(prm);
			
			// Check if any indents are different
			if(ind !== mainIndent)
				mainIndent = -1;
			if(ind != 0)
				hasIndent = true;
			if(prm.length != 0)
				hasParams = true;
		}
		// The "Change Face Graphic" command seems to automatically determine the size of the text box. And I think that having an empty string removes any face graphic.
		else if(code === 10130)
			portrait = str !== '';
		else if(other && ((settings.hasOther && str) || (filter && filter.includes(code))))
		{
			other.push({
				path: [...path, i],
				original: str,
				patch: str
			});
		}
		
		last = code;
	}
}

function extractDialogue(patch, isDatabase)
{
	let orig = '';
	let p = '';
	let path1;
	let path2;
	
	if(patch.dialogue)
	{
		for(let entry of patch.dialogue)
		{
			if(entry.path[0] !== path1 || (!isDatabase && entry.path[1] !== path2))
			{
				orig += '\n';
				p += '\n';
			}
			
			path1 = entry.path[0];
			path2 = entry.path[1];
			orig += (entry.original || '') + '\n';
			p += handleSpecials(entry.lines ? compressLines(entry.lines) : entry.patch) + '\n';
		}
	}
	
	if(patch.other)
	{
		orig += '\n-= Other =-\n\n';
		p += '\n-= Other =-\n\n';
		
		for(let entry of patch.other)
		{
			orig += (entry.original || '') + '\n';
			p += (entry.patch || '') + '\n';
		}
		
		orig = orig.trimEnd();
		p = p.trimEnd();
	}
	
	return (settings.extractPatched ? (orig + '\n\n\n' + p) : orig).trimStart().trimEnd();
}

function applyPatchMap(data, patch)
{
	if(patch.other)
		for(let entry of patch.other)
			data[81][entry.path[0]][5][entry.path[1]][52][entry.path[2]][2] = entry.patch;
	
	if(patch.dialogue)
	{
		// Since Array.splice is a dynamic function, you need to adjust for things that'll change the index.
		let offset = 0;
		// You also need to reset the offset every time you get to a new event list.
		let ev;
		let pg;
		
		for(let entry of patch.dialogue)
		{
			if(ev !== entry.path[0] || pg !== entry.path[1])
				offset = 0;
			ev = entry.path[0];
			pg = entry.path[1];
			
			let commands = getPatchedCommands(entry);
			data[81][entry.path[0]][5][entry.path[1]][52].splice(entry.path[2] + offset, entry.path[3], ...commands);
			offset += commands.length - entry.path[3];
		}
	}
	
	return data;
}

function applyPatchDatabase(data, patch)
{
	if(patch.other)
	{
		for(let entry of patch.other)
		{
			let ref = data;
			let path = entry.path;
			
			for(let i = 0; i < path.length-1; i++)
				ref = ref[path[i]];
			
			ref[path[path.length-1]] = entry.patch;
		}
	}
	
	if(patch.dialogue)
	{
		let offset = 0;
		let ev;
		
		for(let entry of patch.dialogue)
		{
			if(ev !== entry.path[0])
				offset = 0;
			ev = entry.path[0];
			
			let commands = getPatchedCommands(entry);
			data[25][entry.path[0]][22].splice(entry.path[1] + offset, entry.path[2], ...commands);
			offset += commands.length - entry.path[2];
		}
	}
	
	return data;
}

function getPatchedCommands(entry)
{
	let commands = [];
	let lines = entry.lines || splitLine(entry.patch, !!entry.portrait);
	let length = lines.length;
	let ind = entry.indent || [];
	let mainIndent = entry.indent || 0;
	let prm = entry.parameters || [];
	
	if(length > 4 && !settings.suspendErrors)
	{
		console.error(entry);
		throw "Error: The amount of lines for this entry is greater than 4!";
	}
	
	for(let i = 0; i < length; i++)
		commands.push([i === 0 ? 10110 : 20110, ind[i] || mainIndent, lines[i], prm[i] || []]);
	
	return commands;
}

function download(contents, filename = '', override = false)
{
	if(settings.downloadAsZip && !override)
		zip.file(filename, contents);
	else
	{
		scheduler.add(() => {
			const dlink = document.createElement('a');
			dlink.download = filename;
			dlink.href = window.URL.createObjectURL(new Blob([contents]));
			dlink.click();
			dlink.remove();
		}, settings.immediateDownloads ? 1 : settings.delay);
	}
}

function upload(e)
{
	e.preventDefault();
	
	for(let i = 0, list = e.dataTransfer.items, len = list.length; i < len; i++)
	{
		if(list[i].kind === 'file')
		{
			let file = list[i].getAsFile();
			
			if(file.name.includes('.lmu'))
			{
				let reader = new FileReader();
				reader.readAsArrayBuffer(file);
				reader.onload = function()
				{
					let filename = curateName(file.name);
					let map = parseStart(new Uint8Array(this.result), MAP);
					console.log(file.name, 'has been converted. When every file has been processed, execute downloadZipFile() to download the generated zip file.');
					
					if(!settings.disableDownloading)
					{
						let patch = generatePatchMap(map);
						let hasDialogue = Object.keys(patch).length !== 0;
						
						if(hasDialogue || settings.downloadDialoguelessMaps)
							download(JSON.stringify(map), filename + '.json');
						if(hasDialogue)
							download(JSON.stringify(patch), filename + '.patch.json');
					}
				};
			}
			else if(file.name.includes('.ldb'))
			{
				let reader = new FileReader();
				reader.readAsArrayBuffer(file);
				reader.onload = function()
				{
					let db = parseStart(new Uint8Array(this.result), DATABASE);
					console.log(file.name, 'has been converted. When every file has been processed, execute downloadZipFile() to download the generated zip file.');
					chars = {};
					
					for(let c in db[11])
						if(db[11][c][1])
							chars[c] = db[11][c][1];
					
					if(!settings.disableDownloading)
					{
						download(JSON.stringify(db), 'database.json');
						download(JSON.stringify(generatePatchDatabase(db)), 'database.patch.json');
						download(JSON.stringify(chars), 'chars.json');
					}
					
					showCharsEnabled();
				};
			}
			else if(file.name === 'chars.json')
			{
				let reader = new FileReader();
				reader.readAsText(file, 'UTF-8');
				reader.onload = function() {
					chars = JSON.parse(this.result);
					showCharsEnabled();
				};
			}
			else if(file.name.includes('.json'))
			{
				let reader = new FileReader();
				reader.readAsText(file, 'UTF-8');
				reader.onload = function() {stack[file.name] = JSON.parse(this.result)};
			}
		}
	}
}

function handleData()
{
	let mapDialogue = {};
	let text = '';
	
	for(let filename in stack)
	{
		filename = curateName(filename);
		let isDatabase = filename === 'database';
		let data = stack[filename + '.json'];
		let patch = stack[filename + '.patch.json'];
		let hasData = !!data;
		let hasPatch = !!patch;
		let allowSplicing = false;
		
		if(hasPatch)
		{
			if(hasData)
			{
				data = isDatabase ? applyPatchDatabase(data, patch) : applyPatchMap(data, patch);
				allowSplicing = !!patch.allowManualEditing;
			}
			else
			{
				if(settings.downloadBatchedText)
					mapDialogue[filename] = extractDialogue(patch, isDatabase);
				else
					download(extractDialogue(patch, isDatabase), filename + '.txt');
			}
		}
		
		if(hasData)
		{
			if(!settings.disableDownloading)
			{
				// Automated string replacement
				if(hasPatch && !!patch.replace)
				{
					let s = JSON.stringify(data);
					
					for(let key in patch.replace)
						s = s.replace(key, patch.replace[key]);
					
					data = JSON.parse(s);
				}
				
				if(allowSplicing)
				{
					if(isDatabase)
						download(JSON.stringify(data), 'database.json');
					else
						download(JSON.stringify(data), filename + '.json');
				}
				else
				{
					if(isDatabase)
						download(new Uint8Array(createStart(data, DATABASE)), 'RPG_RT.ldb');
					else
						download(new Uint8Array(createStart(data, MAP).concat(0)), filename + '.lmu');
				}
			}
			console.log(hasPatch ? `${filename} was patched.` : `${filename} was not patched. (Or it was manually patched.)`);
		}
		
		delete stack[filename + '.json'];
		delete stack[filename + '.patch.json'];
	}
	
	for(let map in mapDialogue)
		text += "###########\n# " + map + " #\n###########\n" + mapDialogue[map] + '\n\n\n\n';
	
	if(text !== '')
		download(text.trimStart().trimEnd(), 'dump.txt');
}

function downloadZipFile()
{
	zip.generateAsync({type: 'uint8array'}).then((blob) => {
		let size = blob.length;
		let unit = 'byte(s)';
		let divisor;
		
		if(size >= (divisor = 1000000000))
		{
			size = Math.round(size / divisor);
			unit = 'gigabyte(s)';
		}
		else if(size >= (divisor = 1000000))
		{
			size = Math.round(size / divisor);
			unit = 'megabytes(s)';
		}
		else if(size >= (divisor = 1000))
		{
			size = Math.round(size / divisor);
			unit = 'kilobytes(s)';
		}
		
		console.log(`The size of the content you're downloading is around ${size} ${unit}.`);
		download(blob, 'result.zip', true);
		zip = new JSZip();
	});
}

function showCharsEnabled()
{
	document.getElementById('chars').style['background-color'] = 'green';
	document.getElementById('handler').disabled = false;
	document.getElementById('map').disabled = false;
	document.getElementById('load').disabled = false;
}

function curateName(name)
{
	if(name.includes('.patch.json'))
		return name.substring(0, name.lastIndexOf('.patch.json'));
	else
		return name.substring(0, name.lastIndexOf('.'));
}

/*
-= Example =-
copy(JSON.stringify(transpose(stack["Map0148.patch.json"], 12, `Cibon: We finally made it!\\. This city
is called Gencorin, right?

Soko: Overall, we got here a little
faster than expected,\\. despite all
the events on the way.`, {'Kento': '\\c[1]\\n[1]\\c[0]', 'Cibon': '\\c[2]\\n[2]\\c[0]', 'Soko': '\\c[13]\\n[3]\\c[0]'})))

-= Template =-
copy(JSON.stringify(transpose(stack["Map0.patch.json"], 0, {
	'Kento': '\\c[1]\\n[1]\\c[0]',
	'Cibon': '\\c[2]\\n[2]\\c[0]',
	'Soko': '\\c[13]\\n[3]\\c[0]'
}, ``)))
*/
// Returns the patch and modifies it as a side effect of not cloning it. Use with copy().
// Also, this'll inevitably cause any other ':' to be affected. But they're few enough to make it easier to just manually fix that.
function transpose(patch, offset, dictionary, text)
{
	if(patch)
	{
		if(dictionary)
			for(let key in dictionary)
				text = text.split(key + ':').join(dictionary[key] + ':');
		
		text = text.replace(/([^(\n\\c\[0\]]+?):/g, '\\c[10]$1\\c[0]:');
		text = text.split('\n\n');
		
		for(let i = 0; i < text.length; i++)
			text[i] = text[i].split('\n');
		
		offset = offset || 0;
		
		for(let i = 0; i < text.length; i++)
			patch.dialogue[i + offset].lines = text[i];
		
		return patch;
	}
}

// Transpose the new patch format.
function transposeDynamic(patch, offset, dictionary, text, other)
{
	if(patch)
	{
		if(dictionary)
			for(let key in dictionary)
				text = text.split(key + ':').join(dictionary[key] + ':');
		
		text = text.replace(/\n+/g, '\n');
		text = text.split('\n');
		
		offset = offset || 0;
		
		for(let i = 0; i < text.length; i++)
			patch.dialogue[i + offset].patch = text[i];
		
		if(patch.other)
		{
			other = other.replace(/\n+/g, '\n');
			
			for(let i = 0; i < patch.other.length; i++)
				patch.other[i].patch = other[i];
		}
		
		return patch;
	}
}

// The settings has a function using the current entry as a parameter and returning a boolean based on conditions you choose. It returns true by default.
function convertPatchesFromLegacyFormat()
{
	for(let filename in stack)
	{
		filename = curateName(filename);
		let patch = stack[filename + '.patch.json'];
		
		if(patch && patch.dialogue)
		{
			for(let i = 0, length = patch.dialogue.length; i < length; i++)
			{
				let entry = patch.dialogue[i];
				let replacement = {};
				entry.portrait = entry.portrait === undefined ? settings.hasPortraitCondition(entry) : entry.portrait;
				
				// preserve the order
				let keys = ['path', 'indent', 'parameters', 'portrait', 'original'];
				
				for(let key of keys)
				{
					if(entry[key])
					{
						replacement[key] = entry[key];
						delete entry[key];
					}
					else // if portrait is false, don't include it at all
						delete entry[key];
				}
				
				replacement.patch = compressLines(entry.lines);
				delete entry.lines;
				
				for(let key in entry)
					replacement[key] = entry[key];
				
				patch.dialogue[i] = replacement;
			}
			
			download(JSON.stringify(patch), filename + '.patch.json');
		}
	}
}

// Example //
/*settings.hasPortraitCondition = (entry) => {
	let characters = ['Kento','Cibon','Soko','Saion','Gurim','Jorn'];
	
	for(let c of characters)
		if(entry.original.startsWith(`${c}:`) || entry.original.startsWith(`(${c}:`))
			return true;
	
	return false;
};*/

// There aren't named keys for this reason: Don't expand out what you don't need to expand.
// Leave it as 8 bit arrays unless you actually need to modify what's in it.
// This is first and foremost patching dialogue, not changing the game.
const MAP = {
	strings: [32],
	objects: [81],
	special:
	{
		'81':
		{
			strings: [1],
			objects: [5],
			special:
			{
				'5':
				{
					strings: [21],
					bytecount: 51,
					commands: 52
				}
			}
		}
	}
};

const DATABASE = {
	objects: [11,12,13,14,15,16,17,18,19,20,23,24,25],
	lists: [21,22],
	special:
	{
		'11': {strings: [1,2,3,15,67]},
		'12': {strings: [1]},
		'13': {strings: [1,2]},
		'14': {strings: [1,2]},
		'15': {strings: [1]},
		'16': {strings: [1]},
		'17': {strings: [1]},
		'18': {strings: [1,51,52,55]},
		'19': {strings: [1]},
		'20': {strings: [1,2]},
		'21': {strings: '*'},
		'22': {},
		'23': {strings: [1]},
		'24': {strings: [1]},
		'25':
		{
			strings: [1],
			bytecount: 21,
			commands: 22
		}
	}
};