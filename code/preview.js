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