# Foreword

Whilst designing the language, I made the following rule for myself:

> A programming language should ideally make it easy to write clear descriptive unambiguous code, and difficult to write
code that isn't those things

Another thing which I tried to keep in mind, which I think bears repeating, is the following:

> Code is more often read than it is written

In short, I wanted to create a language that is both easy to write in, but also easy to read. There's this thing in 
called the "no free lunch theorem", which, in this context, basically means that there is no perfect solution to any
problem. Each potential advantage comes with its own set of disadvantages. It is up to the designer to weigh these 
trade-offs and make a decision. This also holds true for language design. 

* Increased flexibility makes it easier for a developer to express their ideas
* At the same time, this increased flexibility could make code harder to read for other developers as there suddenly are
  more ways to do the same thing
* A strongly typed language is generally considered safer
* But, explicit types can decrease flexibility and increase development time
* Explicit error handling makes developers more aware of what might go wrong and what will happen in those situations
* However, the increased verbosity might...

And the list just keeps on growing. Language design is tricky to say the least. I often see discussions online about 
what the supposed "best" programming language is. I think this misses the point wholly. A language, really, is just a
tool to solve problems. Just like tools in a toolbox, certain languages are better suited for certain jobs. I could
tighten a screw with a hammer, but if I end up failing, that isn't really the fault of the hammer, that's on me for 
picking the wrong tool. Were I to work on an embedded system, pointers in Rust or C/C++ would be indispensable, but were
I to work in a more mathematical domain, languages such as python or Haskell would be more well suited. 

The problems being solved also change over time. I can't imagine that the computer scientists of the 70's and 80's
could've imagined that computers would become as wide-spread as they are today. As of writing this, in most developed
countries, computers have become ubiquitous. Everyone seems has in their pocket what not too long ago would've been
called a supercomputer. Machine learning has taken a foothold in most major industries, and the internet of things is
starting to take off.

I will say that certain languages features, no matter how well intentioned they may have been, or insignificant they 
seemed at the time, may not have been the best choice in hindsight. Lack of bounds checking can lead to buffer
overflows, lack of null safety leads to undefined behaviour, and so on. But hindsight is 20/20 as they say. As the 
saying goes:

> A tool is either not used, or complained about

Still, over time, I think that we have gotten better at designing languages. What have been found to be common mistakes
can be discouraged through language design. Common language patterns might be taken into account when designing a
language, allowing for more elegant idiomatic solutions. Personally, when talking about general purpose languages (as
opposed to more domain specific languages), I believe we should stave off the temptation to add too many language 
features. A certain balance should be kept I believe, especially within the domain of problems the language aims to
solve, or the language might start to feel asymmetric. A certain feature might allow very specific problems to be solved
idiomatically, but then we run into issues:

* If I add a language feature to solve problem A, why should I not add one to more idiomatically solve problem B, and
  what about problem C?
* Every new feature added to a language makes it more difficult to learn a language, and make it more difficult to
  review code written in said language
* Every new feature added has the potential to add yet another way to solve the same problem

No language is perfect, including this one, and I think that it will, at least for the time being, be impossible to
design a perfect language. People and problems change over time, and different domains have different requirements.
Nevertheless, I always think that it's worth to try to improve upon past decisions, and see where that might lead us.

Joël Abrahams, 2019
