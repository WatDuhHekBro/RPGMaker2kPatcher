function upload(f)
{
	if(f)
	{
		let reader = new FileReader();
		reader.onload = function() {
			parseMain(new Uint8Array(this.result));
			let filename = f.name.substring(0, f.name.indexOf('.lmu'));
			let patch = generatePatch(map);
			download(JSON.stringify(map), filename + '.json');
			download(JSON.stringify(patch), filename + '.patch.json');
			download(extractDialogue(patch), filename + '.txt');
		};
		reader.readAsArrayBuffer(f);
	}
}

function parseMain(b)
{
	let main = new ByteReader(b);
	map = {header: main.readString()};
	while(stepMain(map, main));
	console.log(map);
}

function parseEvents(b)
{
	let ev = {};
	
	for(let length = b.readInt(); length--;)
	{
		let id = b.readInt().toString();
		let event = {};
		while(stepEvent(event, b));
		ev[id] = event;
	}
	
	return ev;
}

function parsePages(b)
{
	let pages = {};
	
	for(let length = b.readInt(); length--;)
	{
		let id = b.readInt().toString();
		let page = {};
		while(stepPage(page, b));
		delete page[getKeyFromValue(PAGE, 51)];
		pages[id] = page;
	}
	
	return pages;
}

function parseCommands(b)
{
	let commands = [];
	let event;
	let condition = false;
	
	do
	{
		if(condition)
			commands.push(event);
		event = b.readEvent();
		condition = event[0] != 0;
	}
	while(condition);
	
	return commands;
}

function stepMain(a, b)
{
	let id = b.readInt();
	let val;
	
	if(id === 81)
		val = parseEvents(b.spliceBytes());
	else if(id === 71 || id === 72)
		val = b.readMap();
	else if(id === 32)
		val = b.readString();
	else if(id)
		val = b.readStaticInt();
	
	if(id && val !== undefined)
		a[getKeyFromValue(MAIN, id)] = val;
	
	return id;
}

function stepEvent(a, b)
{
	let id = b.readInt();
	let val;
	
	if(id === 5)
		val = parsePages(b.spliceBytes());
	else if(id === 1)
		val = b.readString();
	else if(id)
		val = b.readStaticInt();
	
	if(id && val !== undefined)
		a[getKeyFromValue(EVENT, id)] = val;
	
	return id;
}

function stepPage(a, b)
{
	let id = b.readInt();
	let val;
	
	if(id === 52)
		val = parseCommands(b.spliceBytes());
	else if(id === 2 || id === 41)
		val = b.read8bIntArray();
	else if(id === 21)
		val = b.readString();
	else if(id)
		val = b.readStaticInt();
	
	if(id && val !== undefined)
		a[getKeyFromValue(PAGE, id)] = val;
	
	return id;
}