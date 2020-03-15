// Have one ByteWriter per process that needs a byte count. In total, 4 types per map: main, events, pages, and commands.
function createMain(data)
{
	let main = new ByteWriter();
	main.writeString(data.header);
	delete data.header;
	
	for(let key in data)
	{
		let id = MAIN[key] || parseInt(key);
		let val = data[key];
		main.writeInt(id);
		
		if(id === 81)
			main.merge(createEvents(val));
		else if(id === 71 || id === 72)
			main.writeMap(val);
		else if(id === 32)
			main.writeString(val);
		else
			main.writeStaticInt(val);
	}
	
	main.write8bInt(0);
	
	return main.bytes;
}

function createEvents(data)
{
	let events = new ByteWriter();
	events.writeInt(Object.keys(data).length);
	
	for(let e in data)
	{
		let ev = data[e];
		events.writeInt(Number(e));
		
		for(let property in ev)
		{
			let id = EVENT[property] || parseInt(property);
			let val = ev[property];
			events.writeInt(id);
			
			if(id === 5)
				events.merge(createPages(val));
			else if(id === 1)
				events.writeString(val);
			else
				events.writeStaticInt(val);
		}
		
		events.write8bInt(0);
	}
	
	return [...events.getByteCount(), ...events.bytes];
}

function createPages(data)
{
	let pages = new ByteWriter();
	pages.writeInt(Object.keys(data).length);
	
	for(let p in data)
	{
		let pg = data[p];
		pages.writeInt(Number(p));
		
		for(let property in pg)
		{
			let id = PAGE[property] || parseInt(property);
			let val = pg[property];
			let cmds;
			
			if(id === 52)
			{
				val = createCommands(val);
				cmds = true;
			}
			else
				pages.writeInt(id);
			
			if(cmds)
			{
				pages.writeInt(51);
				pages.writeStaticInt(val.bytes.length);
				pages.writeInt(id);
				pages.merge(val.getByteCount(), val.bytes);
			}
			else if(id === 2 || id === 41)
				pages.write8bIntArray(val);
			else if(id === 21)
				pages.writeString(val);
			else
				pages.writeStaticInt(val);
		}
		
		pages.write8bInt(0);
	}
	
	return [...pages.getByteCount(), ...pages.bytes];
}

function createCommands(data)
{
	let commands = new ByteWriter();
	
	for(let e of data)
		commands.writeEvent(e);
	commands.writeEvent([0, 0, '', []]);
	
	return commands;
}