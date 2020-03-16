let stack = {};
let disableDownloading = false;

function generatePatchMap(data, hasOther = false)
{
	let dialogue = [];
	let other = [];
	let e = data[81];
	
	for(let ev in e)
	{
		let ev_num = Number(ev);
		let p = e[ev][5];
		
		for(let pg in p)
		{
			let pg_num = Number(pg);
			generatePatchEvent(dialogue, p[pg][52], [ev_num, pg_num], hasOther ? other : undefined);
		}
	}
	
	return hasOther ? {dialogue: dialogue, other: other} : {dialogue: dialogue};
}

// Vocabulary: path = [global_id, term #]
// Common Events Dialogue: path = [global_id, event #, pos, length]
function generatePatchDatabase(data)
{
	let vocab = [];
	let dialogue = [];
	let e = data[21];
	
	for(let term in e)
	{
		vocab.push({
			path: [21, id],
			patch: e[id]
		});
	}
	
	e = data[25];
	
	for(let ev in e)
	{
		let ev_num = Number(ev);
		generatePatchEvent(dialogue, e[ev][22], [25, ev_num]);
	}
	
	return {vocabulary: vocab, dialogue: dialogue};
}

function generatePatchEvent(patch, cmd, path, other)
{
	// 10110 as A
	// 20110 as B
	// A o --> [A]
	// A A --> [A] [A]
	// A B --> [A,B]
	// A B B --> [A,B,B]
	// A B A --> [A,B] [A]
	for(let i = 0, len = cmd.length, last = null, lines = [], indent = [], params = [], hasIndent = false, hasParams = false, start = 0; i < len; i++)
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
				entry.indent = ind;
			if(hasParams)
				entry.parameters = prm;
			entry.lines = lines;
			patch.push(entry);
			lines = [];
			indent = [];
			params = [];
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
			if(ind != 0)
				hasIndent = true;
			if(prm.length != 0)
				hasParams = true;
		}
		else if(other && str)
		{
			other.push({
				path: [...path, i],
				patch: str
			});
		}
		
		last = code;
	}
}

function extractDialogue(patch)
{
	let s = '';
	
	for(let entry of patch.dialogue)
	{
		for(let line of entry.lines)
			s += line.replace(/\\[^n](\[\d*\])*/g, '').trimEnd() + ' ';
		s = s.trimEnd() + '\n';
	}
	
	return s.trimEnd();
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
			let commands = [];
			
			for(let i = 0, lines = entry.patch, len = lines.length; i < len; i++)
				commands.push([i === 0 ? 10110 : 20110, 0, lines[i], []]);
			
			data[81][entry.path[0]][5][entry.path[1]][52].splice(entry.path[2] + offset, entry.path[3], ...commands);
			offset += entry.patch.length - entry.path[3];
		}
	}
	
	return data;
}

function applyPatchDatabase(data, patch)
{
	if(patch.vocabulary)
		for(let entry of patch.vocabulary)
			data[entry.path[0]][entry.path[1]] = entry.patch;
	
	if(patch.dialogue)
	{
		let offset = 0;
		let ev;
		
		for(let entry of patch.dialogue)
		{
			if(ev !== entry.path[1])
				offset = 0;
			
			ev = entry.path[1];
			let commands = [];
			
			for(let i = 0, lines = entry.patch, len = lines.length, ind = entry.indent || [], prm = entry.parameters || []; i < len; i++)
				commands.push([i === 0 ? 10110 : 20110, ind[i] || 0, lines[i], prm[i] || []]);
			
			data[entry.path[0]][entry.path[1]].splice(entry.path[2] + offset, entry.path[3], ...commands);
			offset += entry.patch.length - entry.path[3];
		}
	}
}

function checkDialogue(patch)
{
	for(let entry of patch.dialogue)
	{
		for(let line of entry.patch)
		{
			let chars = line.replace(/\\[^n](\[\d*\])*/g, '').length;
			let text_safe = chars <= 50;
			let portrait_safe = chars <= 38;
			console.log(`%c-= Line Analysis =-\nLine: %c${line}\n%cCharacters: %c${chars}\n%cText Safe? %c${text_safe}\n%cPortrait Safe? %c${portrait_safe}`, 'color: black', 'color: #800000', 'color: black', 'color: #e18000', 'color: black', text_safe ? 'color: green' : 'color: red', 'color: black', portrait_safe ? 'color: green' : 'color: red');
		}
	}
}

function download(contents, filename = '')
{
	const dlink = document.createElement('a');
	dlink.download = filename;
	dlink.href = window.URL.createObjectURL(new Blob([contents]));
	dlink.click();
	dlink.remove();
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
					console.log(map);
					let patch = generatePatchMap(map);
					
					if(!disableDownloading)
					{
						download(JSON.stringify(map), filename + '.json');
						download(JSON.stringify(patch), filename + '.patch.json');
						download(extractDialogue(patch), filename + '.txt');
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
					console.log(db);
					let patch = generatePatchDatabase(db);
					
					if(!disableDownloading)
					{
						download(JSON.stringify(db), 'database.json');
						download(JSON.stringify(patch), 'database.patch.json');
						download(extractDialogue(patch), 'database.txt');
					}
				};
			}
			else if(file.name.includes('.json'))
			{
				let reader = new FileReader();
				reader.readAsText(file, 'UTF-8');
				reader.onload = function() {stack[file.name] = JSON.parse(this.result)};
			}
			else
			{
				let reader = new FileReader();
				reader.readAsArrayBuffer(file);
				reader.onload = function() {stack[file.name] = new Uint8Array(this.result)};
			}
		}
	}
}

function handleData()
{
	for(let filename in stack)
	{
		filename = curateName(filename);
		
		// MAP --> DATABASE, .lmu --> .ldb, applyPatch
		/*if(filename === 'database')
		{
			
		}
		else
		{
			
		}*/
		
		let map = stack[filename + '.json'];
		let patch = stack[filename + '.patch.json'];
		let hasMap = !!map;
		let hasPatch = !!patch;
		
		if(hasPatch)
		{
			if(hasMap)
				map = applyPatchMap(map, patch);
			else
				checkDialogue(patch);
		}
		
		if(hasMap)
		{
			download(new Uint8Array(createStart(map, MAP)), filename + '.lmu');
			console.log(hasPatch ? `${filename} was patched.` : `${filename} was not patched.`);
		}
		
		delete stack[filename + '.json'];
		delete stack[filename + '.patch.json'];
	}
}

function getKeyFromValue(definitions, value)
{
	let v;
	
	for(let key in definitions)
		if(definitions[key] === value)
			v = key;
	
	if(!v)
		v = value + '?';
	
	return v;
}

function getDefinitionFromValue(definitions, value)
{
	let v;
	
	for(let key in definitions)
		if(definitions[key].id === value)
			v = key;
	
	if(!v)
		v = value + '?';
	
	return v;
}

function curateName(name)
{
	if(name.includes('.patch.json'))
		return name.substring(0, name.lastIndexOf('.patch.json'));
	else
		return name.substring(0, name.lastIndexOf('.'));
}

/*const MAIN = {
	'chipset': 1,
	'width': 2,
	'height': 3,
	'scroll': 11,
	'parallax': 31,
	'parallax_name': 32,
	'parallax_horizontal': 33,
	'parallax_vertical': 34,
	'parallax_horizontal_auto': 35,
	'parallax_horizontal_auto_speed': 36,
	'parallax_vertical_auto': 37,
	'parallax_vertical_auto_speed': 38,
	'lower': 71,
	'upper': 72,
	'events': 81,
	'save': 91
};

const EVENT = {
	'name': 1,
	'x': 2,
	'y': 3,
	'pages': 5 
};

const PAGE = {
	'condition': 2,
	'charset': 21,
	'charset_other': 22,
	'charset_dir': 23,
	'translucent': 24,
	'charset_index': 25,
	'move_type': 31,
	'move_frequency': 32,
	'trigger': 33,
	'draw_priority': 34,
	'draw_prevent_conflict': 35,
	'anim_type': 36,
	'move_route': 41,
	'event_size': 51,
	'event': 52
};

const TYPE = {
	INT: 0,
	INT_STATIC: 1,
	INT8: 2,
	INT16: 3,
	STRING: 4,
	INT_ARRAY: 5,
	INT8_ARRAY: 6,
	COMMANDS: 7,
	TILES: 8,
	LIST: 9,
	OBJECT: 10
};*/

/*const MAP = {
	chipset:
	{
		id: 1,
		type: TYPE.INT_STATIC
	},
	width:
	{
		id: 2,
		type: TYPE.INT_STATIC
	},
	height:
	{
		id: 3,
		type: TYPE.INT_STATIC
	},
	scroll:
	{
		id: 11,
		type: TYPE.INT_STATIC
	},
	parallax:
	{
		id: 31,
		type: TYPE.INT_STATIC
	},
	parallax_name:
	{
		id: 32,
		type: TYPE.STRING
	},
	parallax_horizontal:
	{
		id: 33,
		type: TYPE.INT_STATIC
	},
	parallax_vertical:
	{
		id: 34,
		type: TYPE.INT_STATIC
	},
	parallax_horizontal_auto:
	{
		id: 35,
		type: TYPE.INT_STATIC
	},
	parallax_horizontal_auto_speed:
	{
		id: 36,
		type: TYPE.INT_STATIC
	},
	parallax_vertical_auto:
	{
		id: 37,
		type: TYPE.INT_STATIC
	},
	parallax_vertical_auto_speed:
	{
		id: 38,
		type: TYPE.INT_STATIC
	},
	lower:
	{
		id: 71,
		type: TYPE.TILES
	},
	upper:
	{
		id: 72,
		type: TYPE.TILES
	},
	events:
	{
		id: 81,
		type: TYPE.OBJECT,
		value:
		{
			name:
			{
				id: 1,
				type: TYPE.STRING
			},
			x:
			{
				id: 2,
				type: TYPE.INT_STATIC
			},
			y:
			{
				id: 3,
				type: TYPE.INT_STATIC
			},
			pages:
			{
				id: 5,
				type: TYPE.OBJECT,
				value:
				{
					condition:
					{
						id: 2,
						type: TYPE.INT8_ARRAY
					},
					charset:
					{
						id: 21,
						type: TYPE.STRING
					},
					charset_other:
					{
						id: 22,
						type: TYPE.INT_STATIC
					},
					charset_dir:
					{
						id: 23,
						type: TYPE.INT_STATIC
					},
					translucent:
					{
						id: 24,
						type: TYPE.INT_STATIC
					},
					charset_index:
					{
						id: 25,
						type: TYPE.INT_STATIC
					},
					move_type:
					{
						id: 31,
						type: TYPE.INT_STATIC
					},
					move_frequency:
					{
						id: 32,
						type: TYPE.INT_STATIC
					},
					trigger:
					{
						id: 33,
						type: TYPE.INT_STATIC
					},
					draw_priority:
					{
						id: 34,
						type: TYPE.INT_STATIC
					},
					draw_prevent_conflict:
					{
						id: 35,
						type: TYPE.INT_STATIC
					},
					anim_type:
					{
						id: 36,
						type: TYPE.INT_STATIC
					},
					move_route:
					{
						id: 41,
						type: TYPE.INT8_ARRAY
					},
					event_size:
					{
						id: 51,
						type: TYPE.INT_STATIC,
						skip: true
					},
					event:
					{
						id: 52,
						type: TYPE.COMMANDS,
						extra: 'event_size'
					}
				}
			}
		}
	},
	save:
	{
		id: 91,
		type: TYPE.INT_STATIC
	}
};*/

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