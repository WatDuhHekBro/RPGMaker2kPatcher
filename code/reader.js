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
	let val;
	
	if(id)
	{
		if(ByteReader.bytes[reader.pos] == 0)
		{
			val = null;
			reader.pos++;
		}
		else if('strings' in definitions && (definitions.strings === '*' || definitions.strings.includes(id)))
			val = reader.readString();
		else if('objects' in definitions && definitions.objects.includes(id))
			val = parseObject(reader.spliceBytes(), definitions.special[id]);
		else if('lists' in definitions && definitions.lists.includes(id))
			val = parseList(reader.spliceBytes(), definitions.special[id])
		else if(id === definitions.commands)
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
		else if(id === definitions.bytecount)
			reader.readIntStatic();
		else
			val = reader.readInt8Array();
		
		if(val !== undefined)
			data[id] = val;
	}
	
	console.log('Key', id, 'added with a value of', val, 'at position', reader.pos, 'with an endpoint of', reader.end);
	
	return id;
}