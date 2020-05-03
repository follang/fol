<p align="center">
    <img alt="logo" src="/etc/logo.svg" width="300px">
</p>


<a href="http://www.follang.org"></a><h2><p align="center">www.follang.org</p></h2></a>

<p align="center">
  <a href="https://github.com/follang/fol/blob/develop/LICENSE.md"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://travis-ci.org/follang/fol"><img alt="Travis (.org)" src="https://img.shields.io/travis/follang/fol"></a>
  <a href="https://codecov.io/github/follang/fol"><img alt="Codecov" src="https://img.shields.io/codecov/c/github/follang/fol"></a>
  <a href="https://gitter.im/follang/community"><img alt="Gitter" src="https://img.shields.io/gitter/room/bresilla/follang"></a>
  <a href="https://github.com/follang/fol/blob/develop/.all-contributorsrc"><img src="https://img.shields.io/badge/all_contributors-1-orange.svg" alt="Contributors"></a>
</p>

<p align="center">general-purpose and systems programming language</p>
<hr>


FOL is a general-purpose, systems programming language designed for robustness, efficiency, portability, expressiveness and most importantly elegance. Heavily inspired (and shamelessly copying) from languages: zig, nim, c++, go, rust, julia (in this order), hence the name - FOL (Frankenstein's Objective Language). In Albanian language "fol" means "speak".

<p align="center">  ** FOL IS STILL JUST AN IDEA **  </p>

<hr>

## BUILDING BLOCKS


__*Everything*__ in **FOL** is declared like below:

```
	declaration<options> name: returntype = { body; };
	declaration<options> name: returntype = { body; } | { checker } | { alternative; };
```


#### four top-most declarations are:
```
	def<>		// preporcesr, iports, includes, macros, bocks, definitions ...
	var<>		// all variables, ints, strings, bools, arrays, vecotrs ...
	typ<>  		// new types, structs, objects, interfaces, enums ...
	fun<>		// all methods, functions, rutines and subrutines ...
```
#### a control flow and keywords:
```
	if(condition){} orif(condition){} else{};
	loop(condition){};
	for(){};
	each(){};
	case(variable){like(){}; like(){}; else{}};
	case(variable){type(){}; type(){}; else{}};
	jump();
	continue; break; return; yeild;

```

#### keywords:

```
	result;			// the default return type of function (already made when a subroutine is created)
	default;		// a default value for function (in case of error or unreachable)
	error;			// each routine carries an error variable and can be checked if was raised during call
	check();		// checking a function for error
```

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore -->
<table>
  <tr>
    <td align="center"><a href="http://www.bresilla.com"><img src="https://avatars0.githubusercontent.com/u/10802141?v=4" width="100px;" alt="Trim Bresilla"/><br /><sub><b>Trim Bresilla</b></sub></a><br /><a href="#infra-bresilla" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="https://github.com/follang/fol/commits?author=bresilla" title="Tests">⚠️</a> <a href="https://github.com/follang/fol/commits?author=bresilla" title="Code">💻</a></td>
  </tr>
</table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
