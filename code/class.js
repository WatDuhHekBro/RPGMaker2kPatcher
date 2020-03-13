class ByteReader
{
	constructor(b)
	{
		this.bytes = b;
		this.pos = 0;
	}
	
	spliceBytes()
	{
		let length = this.readInt();
		let reader = new ByteReader(this.bytes.slice(this.pos, this.pos + length));
		this.pos += length;
		return reader;
	}
	
	readMap()
	{
		let values = [];
		
		for(let length = this.readInt()/2; length--;)
			values.push(this.readLE16bInt());
		
		return values;
	}
	
	readEvent()
	{
		let code = this.readInt(),
		    indent = this.readInt(),
		    string = this.readString(),
		    parameters = [];
		
		for(let length = this.readInt(); length--;)
			parameters.push(this.readInt());
		
		return [code, indent, string, parameters];
	}
	
	// Defined Length
	readString()
	{
		let s = '';
		
		for(let length = this.readInt(); length--;)
			s += String.fromCharCode(this.read8bInt());
		
		return s;
	}
	
	// Defined Length
	readIntArray()
	{
		let array = [];
		
		for(let length = this.readInt(); length--;)
			array.push(this.readInt());
		
		return array;
	}
	
	// Defined Length
	read8bIntArray()
	{
		let array = [];
		
		for(let length = this.readInt(); length--;)
			array.push(this.read8bInt());
		
		return array;
	}
	
	// Defined Length
	readStaticInt()
	{
		let digits = [];
		
		for(let length = this.readInt(); length--;)
			digits.push(this.read8bInt());
		
		return ByteReader.convertNumber(digits);
	}
	
	readInt()
	{
		let digits = [];
		
		for(let length = 5; length--;)
		{
			let d = this.bytes[this.pos++];
			digits.push(d);
			
			if(d < 0x80)
				break;
		}
		
		return ByteReader.convertNumber(digits);
	}
	
	read8bInt()
	{
		return this.bytes[this.pos++];
	}
	
	readLE16bInt()
	{
		let d1 = this.bytes[this.pos++];
		let d2 = this.bytes[this.pos++];
		return (d2 << 8) + d1;
	}
	
	static convertNumber(digits)
	{
		let sum = 0;
		
		for(let d = digits.length-1, power = 0; d >= 0; d--)
		{
			if(power === 5)
				break;
			
			let num = digits[d];
			num = num >= 0x80 ? (num - 0x80) : num;
			
			if(power === 4 && num >= 0x8)
				sum = (sum + num * Math.pow(0x80, power++)) - 0x100000000;
			else
				sum += num * Math.pow(0x80, power++);
		}
		
		return sum;
	}
}

class ByteWriter
{
	constructor()
	{
		this.bytes = [];
	}
	
	getByteCount()
	{
		return ByteWriter.convertNumber(this.bytes.length);
	}
	
	writeMap(tiles)
	{
		this.writeInt(tiles.length * 2)
		
		for(let tile of tiles)
			this.writeLE16bInt(tile);
	}
	
	writeEvent(e)
	{
		this.writeInt(e[0]);
		this.writeInt(e[1]);
		this.writeString(e[2]);
		this.writeIntArray(e[3]);
	}
	
	// Defined Length
	writeString(str)
	{
		this.writeInt(str.length);
		
		for(let i = 0, len = str.length; i < len; i++)
			this.write8bInt(str.charCodeAt(i));
	}
	
	// Defined Length
	writeIntArray(array)
	{
		for(let i in array)
			array[i] = ByteWriter.convertNumber(array[i]);
		this.merge(array.length, ...array);
	}
	
	// Defined Length
	write8bIntArray(array)
	{
		this.merge(array.length, array);
	}
	
	// Defined Length
	writeStaticInt(num)
	{
		let digits = ByteWriter.convertNumber(num);
		this.merge(digits.length, digits);
	}
	
	writeInt(num)
	{
		this.merge(ByteWriter.convertNumber(num));
	}
	
	write8bInt(num)
	{
		this.bytes.push(num);
	}
	
	writeLE16bInt(num)
	{
		num = num.toString(16).padStart(4, '0');
		this.merge(parseInt(num.slice(2,4), 16), parseInt(num.slice(0,2), 16));
	}
	
	merge(...args)
	{
		this.bytes = this.bytes.concat(...args);
	}
	
	static convertNumber(num = 0)
	{
		let digits = [0x80, 0x80, 0x80, 0x80, 0x0];
		let negative = num < 0;
		
		if(negative)
		{
			num += 0x100000000;
			
			if(Math.floor(num / 0x80000000) > 1)
				throw `Number ${num} is out of the range of a signed 32 bit integer!`;
			
			num %= 0x80000000;
			digits[0] += 0x8;
		}
		
		for(let d = 0, power = 4; d < 5; d++)
		{
			let p = Math.pow(0x80, power--);
			digits[d] += Math.floor(num / p);
			num %= p;
		}
		
		for(let d = 0; d < 5; d++)
		{
			if(digits[d] !== 0x80)
			{
				digits = digits.slice(d);
				break;
			}
		}
		
		return digits;
	}
}