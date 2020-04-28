// The Problem: I want to be able to dynamically schedule events using setTimeout, however, setTimeout doesn't have this scheduling aspect.
class Scheduler
{
	// Initializes the scheduler and declares the variables.
	// activated (boolean): Whether it continues to run.
	// stack (array): Array of functions to be executed.
	// delays (array): Array of delays in milliseconds.
	// pointer (number): The index of the current function that is being executed.
	constructor()
	{
		this.reset();
	}
	
	// Executes a function off the stack. Since it's automatic, this should not be touched, and doing so will mess with the entire thing (it'll probably execute in two lines in parallel).
	execute()
	{
		// In order to keep track of IDs numerically, functions aren't actually deleted, at least not until all functions are executed.
		if(this.pointer >= this.stack.length)
			this.reset();
		else if(this.activated)
		{
			setTimeout(function() {
				let call = this.stack[this.pointer++];
				call && call();
				this.execute();
			}.bind(this), this.delays[this.pointer] || 500);
		}
	}
	
	// Returns an ID which you can cancel. Also starts the scheduler if it's stopped.
	add(action, delay = 500)
	{
		if(action && delay)
		{
			this.stack.push(action);
			this.delays.push(delay);
		}
		
		this.run();
		return this.stack.length-1;
	}
	
	// Returns a boolean on the success of the removal.
	cancel(id)
	{
		if(id > this.pointer)
		{
			this.stack[id] = () => {};
			this.delays[id] = () => {};
			return true;
		}
		else
			return false;
	}
	
	// Optional time parameter, or run indefinitely.
	run(time)
	{
		if(!this.activated)
		{
			this.activated = true;
			this.execute();
		}
		
		if(time)
			setTimeout(() => {this.activated = false}, time);
	}
	
	// Optional time parameter, or pause it indefinitely.
	pause(time)
	{
		this.activated = false;
		
		if(time)
		{
			setTimeout(() => {
				this.activated = true;
				this.execute();
			}, time);
		}
	}
	
	reset()
	{
		this.activated = false;
		this.stack = [];
		this.delays = [];
		this.pointer = 0;
	}
}