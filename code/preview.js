'use strict';

const PREVIEW = document.getElementById('preview');

/*
<div class="dialogue">
	<code class="check"></code>
	<div class="lines">
		<code></code>
		<code></code>
		<code></code>
		<code></code>
	</div>
	<code class="overflow"></code>
</div>
*/
function createDialogueElement()
{
	let dialogue = document.createElement('div');
	dialogue.classList.add('dialogue');
	let check = document.createElement('code');
	check.classList.add('check');
	let lines = document.createElement('div');
	lines.classList.add('lines');
	let overflow = document.createElement('overflow');
	overflow.classList.add('overflow');
	
	for(let amount = 4; amount--;)
		lines.appendChild(document.createElement('code'));
	
	dialogue.appendChild(check);
	dialogue.appendChild(lines);
	dialogue.appendChild(overflow);
	
	return dialogue;
}

function setPreviewLength(length = 0)
{
	if(length < 0)
		length = 0;
	
	while(PREVIEW.firstChild)
		PREVIEW.removeChild(PREVIEW.firstChild);
	
	for(;length--;)
		PREVIEW.appendChild(createDialogueElement());
}

// I decided to not include colors because they're way too much work for too little gain. The important part here is to check for line endings and if they look nice, not replicating text, which can just be done in-game.
// entry.lines always overrides entry.patch since it's more specific.
function setDisplay(set, line, portrait = false, overrideLines)
{
	if(!chars)
	{
		setPreviewLength(0);
		let message = document.createElement('p');
		message.innerHTML = "<code>chars.json</code> is required!";
		PREVIEW.appendChild(message);
	}
	else if(set)
	{
		let length = portrait ? 38 : 50;
		
		if(overrideLines)
		{
			set.children[0].innerText = ''.padEnd(length, String.fromCharCode(0xA0));
			
			for(let i = 0, display = set.children[1].children; i < display.length; i++)
			{
				display[i].innerText = handleSpecials(overrideLines[i] || '');
				display[i].innerText += ''.padEnd(length - display[i].innerText.length, String.fromCharCode(0xA0));
			}
			
			set.children[2].innerText = overrideLines.slice(4).join(' ');
		}
		else if(line)
		{
			let lines = splitLine(line, portrait, false);
			set.children[0].innerText = ''.padEnd(length, String.fromCharCode(0xA0));
			
			for(let i = 0, display = set.children[1].children; i < display.length; i++)
			{
				display[i].innerText = handleSpecials(lines[i]);
				display[i].innerText += ''.padEnd(length - display[i].innerText.length, String.fromCharCode(0xA0));
			}
			
			set.children[2].innerText = handleSpecials(lines[4]);
		}
	}
}

// http://www.yanfly.moe/wiki/Category:Text_Codes_(MV)
// https://www.francelettekindnessadventure.com/color-codes---rpg-maker.html
/*
\c[2]Cibon\c[0]: Not really.\. I just wanted to tear you away from your little nap!
- '\' detected, 'c' detected --> '[' start of number until ']'
- 'Cibon' --> length = 5
- repeat #1
- ': Not really.' --> length = 18
- '\' detected, '.' detected --> skip (unless it's '\\')
- ' I just wanted to te' --> length = 38 (portrait mode)
*/
// input string, output array of strings
function splitLine(line, portrait = false, trim = true)
{
	let lines = ['','','','',''];
	let index = 0;
	let length = 0; // the actual length of the string
	let space = -1;
	let line_index = 0;
	
	if(!chars)
		throw "Error: Requires chars.json!";
	
	while(index < line.length)
	{
		let c = line[index];
		
		if(c === '\n' && line_index < 4)
		{
			lines[line_index++] = line.substring(0, index);
			line = line.substring(index+1);
			index = 0;
			length = 0;
			space = -1;
		}
		else
		{
			if(c === '\\')
			{
				let action = line[index+1].toLowerCase();
				
				if(['c','i','n','p','s','v'].includes(action))
				{
					let brackets = '';
					let tmp = index+2;
					
					while(line[tmp-1] !== ']')
						brackets += line[tmp++];
					
					// You have to make sure that \\n[#] is counted because of the backslash rule, otherwise, it skews line endings.
					if(action === 'n')
						length += chars[parseInt(brackets.substring(1, brackets.length-1))].length;
					
					index += 2 + brackets.length;
				}
				else
				{
					if(action === '\\')
						length++;
					index += 2;
				}
			}
			else
			{
				if(c === ' ')
					space = index;
				
				index++;
				length++;
				
				if(length > (portrait ? 38 : 50) && line_index < 4)
				{
					let split = space === -1;
					lines[line_index++] = line.substring(0, split ? index-1 : space);
					line = line.substring(split ? index-1 : space+1);
					index = 0;
					length = 0;
					space = -1;
				}
			}
		}
	}
	
	lines[line_index] = line;
	
	if(trim)
	{
		if(lines[4] === '')
		{
			let split = -1;
			
			for(let i = 0; i < lines.length-1; i++)
			{
				if(lines[i] === '')
				{
					split = i;
					break;
				}
			}
			
			lines = lines.slice(0, split);
		}
	}
	
	return lines;
}

function loadDialogue(e)
{
	let map = stack[`Map${e.value.padStart(4,'0')}.patch.json`];
	e.value = '';
	
	if(map && map.dialogue)
	{
		let list = map.dialogue;
		setPreviewLength(list.length);
		
		for(let i = 0; i < list.length; i++)
		{
			let entry = list[i];
			setDisplay(PREVIEW.children[i], entry.patch, !!entry.portrait, entry.lines);
		}
	}
	else
		setPreviewLength(0);
}