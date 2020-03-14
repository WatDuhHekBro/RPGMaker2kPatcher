let map;

function generatePatch(data)
{
	let dialogue = [];
	//let other = [];
	let e = data.events;
	
	for(let ev in e)
	{
		let ev_num = Number(ev);
		let p = e[ev].pages;
		
		for(let pg in p)
		{
			let pg_num = Number(pg);
			
			for(let i = 0, cmd = p[pg].event, len = cmd.length, last = null, lines = [], start = 0; i < len; i++)
			{
				let code = cmd[i][0];
				let str = cmd[i][2];
				
				if((last === 20110 && code !== 20110) || (last === 10110 && (code !== 10110 && code !== 20110)))
				{
					dialogue.push({
						path: [ev_num, pg_num, start, lines.length],
						//original: lines,
						patch: lines
					});
					
					lines = [];
				}
				
				if(code === 10110)
				{
					start = i;
					lines.push(str);
				}
				else if(code === 20110)
					lines.push(str);
				/*else if(str)
				{
					other.push({
						path: [ev_num, pg_num, i],
						original: str,
						patch: ''
					});
				}*/
				
				last = code;
			}
		}
	}
	
	return {dialogue: dialogue/*, other: other*/};
}

function extractDialogue(patch)
{
	let s = '';
	
	for(let entry of patch.dialogue)
	{
		for(let line of entry.patch)
			s += line.replace(/\\.(\[\d*\])*/g, '').trimEnd() + ' ';
		s = s.trimEnd() + '\n';
	}
	
	return s.trimEnd();
}

function applyPatch(data, patch)
{
	for(let entry of patch.other)
		if(entry.patch)
			data.events[entry.path[0]].pages[entry.path[1]].event[entry.path[2]][2] = entry.patch;
	
	for(let entry of patch.dialogue)
	{
		let commands = [];
		
		for(let i = 0, lines = entry.patch, len = lines.length; i < len; i++)
			commands.push([i === 0 ? 10110 : 20110, 0, lines[i], []]);
		
		data.events[entry.path[0]].pages[entry.path[1]].event.splice(entry.path[2], entry.path[3], ...commands);
	}
	
	return data;
}

function checkDialogue(patch)
{
	for(let entry of patch.dialogue)
	{
		for(let line of entry.patch)
		{
			let chars = line.replace(/\\.(\[\d*\])*/g, '').length;
			let text_safe = chars <= 50;
			let portrait_safe = chars <= 38;
			console.log(`%c-= Line Analysis =-\nLine: %c${line}\n%cCharacters: %c${chars}\n%cText Safe? %c${text_safe}\n%cPortrait Safe? %c${portrait_safe}`, 'color: black', 'color: #800000', 'color: black', 'color: #e18000', 'color: black', text_safe ? 'color: green' : 'color: red', 'color: black', portrait_safe ? 'color: green' : 'color: red');
		}
	}
}

/*function applyPatch(data, patch, common)
{
	for(let entry of patch.other)
		if(entry.patch)
			data.events[entry.path[0]].pages[entry.path[1]].event[entry.path[2]][2] = entry.patch;
	
	for(let entry of patch.dialogue)
	{
		if(entry.patch)
		{
			let person = entry.person;
			let text = entry.patch;
			
			if(person)
			{
				let tmp = common.people[person];
				person = tmp ? tmp.patch : person;
				text = person + text;
				person = !!tmp.portrait;
			}
			
			data.events[entry.path[0]].pages[entry.path[1]].event.splice(entry.range[0], (entry.range[1] - entry.range[0]), ...splitDialogue(text));
		}
	}
	
	return data;
}*/

// \c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\.Glaube mir,\....es ist besser so für mich...\.\.\^
// 0: '\', don't count, i++ (extra)
// 2: '[', don't count, start delimiter
// 3: '1', don't count b/c delimiter
// 4: '3', don't count b/c delimiter
// 5: ']', don't count, end delimiter
// ...
/*function splitDialogue(s)
{
	let lines = [];
	let commands = [];
	
	for(let i = 0, len = s.length, chars = 0, delimiter = false, split = 0, last = 0; i < len; i++)
	{
		let c = s[i];
		
		if(c === '\\')
			i++;
		else if(c === '[')
			delimiter = true;
		else if(c === ']')
			delimiter = false;
		else if(!delimiter)
		{
			chars++;
			
			if([' ', '.', ',', '?', '!'].includes(c))
				split = i;
		}
		
		if(chars >= 50)
		{
			lines.push(s.substring(last, split));
			last = split + 1;
			chars = 0;
		}
		else if(i === len-1)
			lines.push(s.substring(last));
	}
	
	if(lines.length > 4)
		console.error(`Line ${s} overflowed past 4 lines!`);
	
	for(let i = 0, len = Math.min(lines.length, 4); i < len; i++)
		commands.push([i === 0 ? 10110 : 20110, 0, lines[i], []]);
	
	return commands;
}*/

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

const MAIN = {
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