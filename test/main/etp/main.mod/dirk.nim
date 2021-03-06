import os, threadpool, strutils

type Dirent* = ref object
    name: string
    path: string
    file: os.FileInfo
    mode: set[FilePermission]
    number*: int
    active*: bool
    select*: bool
    ignore*: bool
    nick: string

method `$`*(dir: Dirent): string {.base.} = dir.path
method `%`*(dir: Dirent): string {.base.} = dir.name
method `~`*(dir: Dirent): string {.base.} = dir.nick

proc makeDir*(path: string): Dirent =
  let dir = normalizedPath(path)
  let name = if extractFilename(dir) == "": "/" else: extractFilename(dir)
  let dirent = Dirent(
    path: dir,
    name: name,
    file: getFileInfo(dir),
    mode: getFilePermissions(dir),
    nick: name)
  return dirent

type Dirents* = seq[Dirent]

proc makeDirs*(paths: varargs[string, `$`]): Dirents =
  for i, value in paths:
    if value == "": continue
    else: result.add(makeDir(value))

method isDir*(this: Dirent): bool {.base.} = dirExists(this.path)
method isFile*(this: Dirent): bool {.base.} = fileExists(this.path)
method isRegular*(this: Dirent): bool {.base.} = this.isDir or this.isFile
method isSymlink*(this: Dirent): bool {.base.} = symlinkExists(this.path)
method isHidden*(this: Dirent): bool {.base.} = this.name[0] == '.'
include help
method getParent*(this: Dirent): Dirent {.base.} = makeDir(parentInfo(this.path))
method getChildren*(this: Dirent): Dirents {.base.} = makeDirs(elementInfo(this.path))
method getSiblings*(this: Dirent): Dirents {.base.} = makeDirs(elementInfo(parentInfo(this.path)))
method getRelatives*(this: Dirent): Dirents {.base.} = makeDirs(elementInfo(parentInfo(parentInfo(this.path))))
method getAncestors*(this: Dirent): Dirents {.base.} = makeDirs(ancestorInfo(parentInfo(this.path)))

proc fileList*(dir: Dirent, recurr = false, ignore = [".git", "node_modules"]): Dirents =
  if not recurr:
    for kind, path in walkDir(dir.path):
      let temp: Dirent = makeDir(path)
      result.add(temp)
  else:
    for path in walker(dir.path, ignoreDirs=ignore):
      let temp: Dirent = makeDir(path)
      result.add(temp)

proc choseFile*(dir: Dirent, incDir = true, incFile = true, incHidden = true, recurrent = false): Dirents =
  if not dir.isDir: return
  var
    paths: Dirents = fileList(dir, recurrent)
    counter: int = 0
  for file in paths:
    file.nick = $file % $dir
    if ((file.isDir and incDir) or (file.isFile and incFile)) and (not file.isHidden or incHidden):
        file.number = counter
        counter.inc
        result.add(file)
  result.sorter(byType)

#var files: Dirents = choseFile(makeDir("/home/bresilla/DATA"), true, true, true, false)
#for file in files: echo file
