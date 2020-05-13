'use strict';

const color = ['ffffff','21a0d7','ff784b','65cb41','99cdfc','ccc0ff','ffff9f','808080','bfbfbf','1f81cc','ff360d','00a110','3e9be0','9f97fe','ffcb1f','000000','84aaff','ffff3d','ff2024','202044','df8040','f3c242','3d7ebe','40bef0','81ff81','c17f80','827fff','fe80fd','02a041','00e05f','a25fe0','c180fe'];
const doclines = [
	document.getElementById('line-0'),
	document.getElementById('line-1'),
	document.getElementById('line-2'),
	document.getElementById('line-3'),
	document.getElementById('line-4'),
	document.getElementById('line-overflow')
];

function setDisplay(set, line, portrait = false)
{
	if(set && line)
	{
		let lines = splitLine(line, portrait);
		set[0].innerHTML = ''.padEnd(portrait ? 38 : 50).split(' ').join(String.fromCharCode(0xA0));
		
		for(let i = 1; i < set.length-1; i++)
			set[i].innerHTML = activateSpecials(lines[i-1]) || String.fromCharCode(0xA0);
		
		set[set.length-1].innerHTML = lines[4];
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
function splitLine(line, portrait = false)
{
	let lines = ['','','','',''];
	let index = 0;
	let length = 0; // the actual length of the string
	let space = -1;
	let line_index = 0;
	
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
				let action = line[index+1];
				
				if(['C','c','I','i','N','n','P','p','V','v'].includes(action))
				{
					let brackets = '';
					let tmp = index+2;
					
					while(line[tmp-1] !== ']')
						brackets += line[tmp++];
					
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
	
	return lines;
}

function activateSpecials(line)
{
	let index = 0;
	let colors = [];
	
	while(index < line.length)
	{
		let c = line[index];
		
		if(c === '\\')
		{
			let action = line[index+1];
			
			if(['C','c','I','i','N','n','P','p','V','v'].includes(action))
			{
				let brackets = '';
				let tmp = index+2;
				
				while(line[tmp-1] !== ']')
					brackets += line[tmp++];
				
				colors.push(parseInt(brackets.substring(1, brackets.length-1)));
				line = line.substring(0, index) + String.fromCharCode(0) + line.substring(index + 2 + brackets.length);
				index++;
			}
			else
			{
				line = line.substring(0, index) + line.substring(action === '\\' ? index+1 : index+2);
				index++;
			}
		}
		else
			index++;
	}
	
	for(let x of colors)
		line = line.replace(String.fromCharCode(0), x === 0 ? '</span>' : `<span style="color:#${color[x]};">`);
	
	return line;
}

setDisplay(doclines, '\\c[13]Cibon\\c[0]: Not really.\\. I just wanted to tear you away from your little nap!', true);