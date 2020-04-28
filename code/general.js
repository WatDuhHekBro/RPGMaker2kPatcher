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

let chars;
let stack = {};
let settings = {
	disableDownloading: false,
	hasOther: false,
	extractPatched: false,
	immediateDownloads: false,
	showReaderLog: false
};
let text = ''; // copy(text) to copy the text-form dialogue.
let scheduler = new Scheduler();

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
			generatePatchEvent(dialogue, p[pg][52], [ev_num, pg_num], other, [10140,20140,10610]);
		}
	}
	
	let out = {dialogue: dialogue};
	
	if(other.length > 0)
		out.other = other;
	
	return out;
}

// Vocabulary: path = [global_id, term #]
// Common Events Dialogue: path = [global_id, event #, pos, length]
function generatePatchDatabase(data)
{
	let dialogue = [];
	let other = [];
	let e = data[25];
	
	for(let ev in e)
	{
		let ev_num = parseInt(ev);
		generatePatchEvent(dialogue, e[ev][22], [ev_num], other, [10140,20140,10610]);
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
function generatePatchEvent(patch, cmd, path, other, filter)
{
	// 10110 as A
	// 20110 as B
	// A o --> [A]
	// A A --> [A] [A]
	// A B --> [A,B]
	// A B B --> [A,B,B]
	// A B A --> [A,B] [A]
	for(let i = 0, len = cmd.length, last = null, lines = [], indent = [], params = [], hasIndent = false, hasParams = false, start = 0, mainIndent = 0; i < len; i++)
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
			entry.original = compressLines(lines);
			entry.lines = lines;
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

function extractDialogue(patch)
{
	let orig = '';
	let p = '';
	let path1;
	let path2;
	
	for(let entry of patch.dialogue)
	{
		if(entry.path[0] !== path1 || entry.path[1] !== path2)
		{
			orig += '\n';
			p += '\n';
		}
		
		path1 = entry.path[0];
		path2 = entry.path[1];
		orig += (entry.original || '') + '\n';
		p += compressLines(entry.lines) + '\n';
	}
	
	return (settings.extractPatched ? (orig + '\n\n\n' + p) : orig).trimStart().trimEnd();
}

function compressLines(lines)
{
	let s = '';
	
	for(let line of lines)
		s += line.replace(/\\[^nv](\[\d*\])*/g, '').trimEnd() + ' ';
	
	for(let c in chars)
		s = s.replace(`\\n[${c}]`, chars[c]);
	
	return s.trimStart().trimEnd();
}

function checkDialogue(patch)
{
	for(let entry of patch.dialogue)
	{
		for(let line of entry.lines)
		{
			line = line.replace(/\\[^nv](\[\d*\])*/g, '');
			for(let c in chars)
				line = line.replace(`\\n[${c}]`, chars[c]);
			let length = line.length;
			let text_safe = length <= 50;
			let portrait_safe = length <= 38;
			console.log(`%c-= Line Analysis =-\nLine: %c${line}\n%cCharacters: %c${length}\n%cText Safe? %c${text_safe}\n%cPortrait Safe? %c${portrait_safe}`, 'color: black', 'color: #800000', 'color: black', 'color: #e18000', 'color: black', text_safe ? 'color: green' : 'color: red', 'color: black', portrait_safe ? 'color: green' : 'color: red');
		}
	}
	
	text = extractDialogue(patch);
	console.log(text);
	console.log("To copy this text, do %ccopy(text)%c.", 'color: #800000', 'color: black');
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
			data[81][entry.path[0]][5][entry.path[1]][52].splice(entry.path[2] + offset, entry.path[3], ...getPatchedCommands(entry));
			offset += entry.lines.length - entry.path[3];
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
			data[25][entry.path[0]][22].splice(entry.path[1] + offset, entry.path[2], ...getPatchedCommands(entry));
			offset += entry.lines.length - entry.path[2];
		}
	}
	
	return data;
}

function getPatchedCommands(entry)
{
	let commands = [];
	
	for(let i = 0, lines = entry.lines, len = lines.length, ind = entry.indent || [], mainIndent = entry.indent || 0, prm = entry.parameters || []; i < len; i++)
		commands.push([i === 0 ? 10110 : 20110, ind[i] || mainIndent, lines[i], prm[i] || []]);
	
	return commands;
}

function download(contents, filename = '')
{
	scheduler.add(() => {
		const dlink = document.createElement('a');
		dlink.download = filename;
		dlink.href = window.URL.createObjectURL(new Blob([contents]));
		dlink.click();
		dlink.remove();
	}, settings.immediateDownloads ? 1 : 250);
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
					console.log(file.name, 'has been converted.');
					
					if(!settings.disableDownloading)
					{
						download(JSON.stringify(map), filename + '.json');
						download(JSON.stringify(generatePatchMap(map)), filename + '.patch.json');
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
					console.log(file.name, 'has been converted.');
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
				};
			}
			else if(file.name === 'chars.json')
			{
				let reader = new FileReader();
				reader.readAsText(file, 'UTF-8');
				reader.onload = function() {chars = JSON.parse(this.result)};
				console.log("Imported chars.json.");
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
				checkDialogue(patch);
		}
		
		if(hasData)
		{
			if(!settings.disableDownloading)
			{
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
}

function curateName(name)
{
	if(name.includes('.patch.json'))
		return name.substring(0, name.lastIndexOf('.patch.json'));
	else
		return name.substring(0, name.lastIndexOf('.'));
}

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