function parseStart(bytes, definitions)
{
	ByteReader.setBytes(bytes);
	let reader = new ByteReader(0, bytes.length);
	let data = {header: reader.readString()};
	while(stepRead(data, reader, definitions));
	return data;
}

function parseObject(reader, definitions)
{
	let data = {};
	
	for(let length = reader.readInt(); length--;)
	{
		let id = reader.readInt().toString();
		let part = {};
		while(stepRead(part, reader, definitions));
		data[id] = part;
	}
	
	return data;
}

function parseList(reader, definitions)
{
	let data = {};
	while(stepRead(data, reader, definitions));
	return data;
}

function stepRead(data, reader, definitions)
{
	let id = reader.readInt();
	let def = getDefinitionFromValue(definitions, id);
	let ins = definitions[def];
	let val;
	
	if(ins)
	{
		if(ins.type === TYPE.INT)
			val = reader.readInt();
		else if(ins.type === TYPE.INT_STATIC)
			val = reader.readIntStatic();
		else if(ins.type === TYPE.INT8)
			val = reader.readInt8();
		else if(ins.type === TYPE.INT16)
			val = reader.readInt16();
		else if(ins.type === TYPE.STRING)
			val = reader.readString();
		else if(ins.type === TYPE.INT_ARRAY)
			val = reader.readIntArray();
		else if(ins.type === TYPE.INT8_ARRAY)
			val = reader.readInt8Array();
		else if(ins.type === TYPE.COMMANDS)
		{
			let b = reader.spliceBytes();
			let commands = [];
			let event;
			let condition = false;
			
			do
			{
				if(condition)
					commands.push(event);
				event = b.readCommand();
				condition = event[0] != 0;
			}
			while(condition);
			
			val = commands;
		}
		else if(ins.type === TYPE.TILES)
			val = reader.readMap();
		else if(ins.type === TYPE.LIST)
			val = parseList(reader.spliceBytes(), ins.value);
		else if(ins.type === TYPE.OBJECT)
			val = parseObject(reader.spliceBytes(), ins.value);
		
		if(ins.skip)
			return null;
	}
	
	if(id)
	{
		// gotta increment even if no definition is found
		if(val === undefined)
			val = reader.readInt8Array();
		data[def] = val;
	}
	
	console.log(`Key "${def}" added with a value of:`, val, `at position ${reader.pos}, endpoint ${reader.end}`);
	
	return id;
}