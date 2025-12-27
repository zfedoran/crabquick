function greet(greeting, punctuation) {
  console.log(greeting + ', ' + this.name + punctuation);
}

const person = { name: 'Alice' };
greet.apply(person, ['Hello', '!']);
