/*function createStart(data, definitions)
{
	let writer = new ByteWriter();
	writer.writeString(data.header);
	delete data.header;
	stepWrite(data, writer, definitions);
	writer.writeInt8(0);
	return writer.bytes;
}

function createObject(data, definitions)
{
	let writer = new ByteWriter();
	writer.writeInt(Object.keys(data).length);
	
	for(let id in data)
	{
		writer.writeInt(parseInt(id));
		stepWrite(data[id], writer, definitions);
		writer.writeInt8(0);
	}
}

function createList(data, definitions)
{
	
}

function stepWrite(data, writer, definitions)
{
	for(let key in data)
	{
		let ins = definitions[key] || parseInt(key);
		let id = ins.id || parseInt();
	}
}*/

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
			main.writeIntStatic(val);
	}
	
	main.writeInt8(0);
	
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
				events.writeIntStatic(val);
		}
		
		events.writeInt8(0);
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
				pages.writeIntStatic(val.bytes.length);
				pages.writeInt(id);
				pages.merge(val.getByteCount(), val.bytes);
			}
			else if(id === 2 || id === 41)
				pages.writeInt8Array(val);
			else if(id === 21)
				pages.writeString(val);
			else
				pages.writeIntStatic(val);
		}
		
		pages.writeInt8(0);
	}
	
	return [...pages.getByteCount(), ...pages.bytes];
}

function createCommands(data)
{
	let commands = new ByteWriter();
	
	for(let e of data)
		commands.writeCommand(e);
	commands.writeCommand([0, 0, '', []]);
	
	return commands;
}