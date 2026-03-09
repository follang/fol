package dirk

import (
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"syscall"
	"time"
)

type Dirent struct {
	name string
	path string
	file os.FileInfo
	mode os.FileMode
	stat *syscall.Stat_t
}

func (d Dirent) IsDir() bool     { return d.mode&os.ModeDir != 0 }
func (d Dirent) IsRegular() bool { return d.mode&os.ModeType == 0 }
func (d Dirent) IsSymlink() bool { return d.mode&os.ModeSymlink != 0 }
func (d Dirent) IsHidden() bool  { return string(d.name[0]) == "." }

type Dirents []*Dirent

func (l Dirents) Len() int           { return len(l) }
func (l Dirents) Less(i, j int) bool { return l[i].name < l[j].name }
func (l Dirents) Swap(i, j int)      { l[i], l[j] = l[j], l[i] }

type File struct {
	T    *Dirent
	File os.FileInfo
	Stat *syscall.Stat_t
	Path string
	Name string

	Number   int
	Active   bool
	Selected bool
	Ignore   bool
	Nick     string

	numLines int
	mapLine  map[int]string
	maxSize  int64
	maxPath  int
}

func MakeFile(dir string) (file File, err error) {
	f, err := os.Stat(dir)
	if err != nil {
		return
	}
	file.T = &Dirent{
		name: filepath.Base(dir),
		path: dir,
		file: f,
		mode: f.Mode(),
		stat: f.Sys().(*syscall.Stat_t),
	}
	file = File{
		File: file.T.file,
		Stat: file.T.stat,
		Name: file.T.name,
		Nick: file.T.name,
		Path: file.T.path,
	}
	return
}

func (f File) IsDir() bool            { return f.File.Mode()&os.ModeDir != 0 }
func (f File) IsRegular() bool        { return f.File.Mode()&os.ModeType == 0 }
func (f File) IsSymlink() bool        { return f.File.Mode()&os.ModeSymlink != 0 }
func (f File) IsHidden() bool         { return string(f.Name[0]) == "." }
func (f File) MimeExte() string       { return getExte(f) }
func (f File) MimeIcon() string       { return getIcon(f) }
func (f File) MimeType() []string     { return getMime(f) }
func (f File) SizeINT(du bool) int64  { return getSize(f, du) }
func (f File) SizeSTR(du bool) string { return byteCountSI(f.SizeINT(du)) }
func (f File) TimeBirth() time.Time   { return timespecToTime(f.Stat.Mtim) }
func (f File) TimeAccess() time.Time  { return timespecToTime(f.Stat.Atim) }
func (f File) TimeChange() time.Time  { return timespecToTime(f.Stat.Ctim) }
func (f File) MaxPath() int           { return f.maxPath }
func (f File) MaxSize() int64         { return f.maxSize }
func (f File) Parent() Files          { return Filer([]string{getParentPath(f)}) }
func (f File) Siblings() Files        { return Filer(elements(getParentPath(f))) }
func (f File) Ancestors() Files       { return Filer(ancestor(getParentPath(f))) }
func (f File) Childrens() Files       { return Filer(elements(f.Path)) }

type Files []*File

func MakeFiles(path []string) (files Files, err error) {
	files = Files{}
	for i := range path {
		if file, err := MakeFile(path[i]); err != nil {
			return files, err
		} else {
			files = append(files, &file)
		}
	}
	return files, nil
}

func Filer(path []string) (files Files) {
	if files, err := MakeFiles(path); err == nil {
		return files
	}
	return files
}

func (e Files) String(i int) string    { return e[i].Name }
func (e Files) Len() int               { return len(e) }
func (e Files) Swap(i, j int)          { e[i], e[j] = e[j], e[i] }
func (e Files) Less(i, j int) bool     { return e[i].Nick[0:] < e[j].Nick[0:] }
func (e Files) SortSize(i, j int) bool { return e[i].SizeINT(DiskUse) < e[j].SizeINT(DiskUse) }
func (e Files) SortDate(i, j int) bool { return e[i].TimeBirth().Before(e[j].TimeBirth()) }

type Element struct {
	sync.RWMutex
	files []*File
}

func (e *Element) Add(item File) {
	e.Lock()
	defer e.Unlock()
	e.files = append(e.files, &item)
}

func fileList(recurrent bool, dir *File) (paths Files, err error) {
	var wg sync.WaitGroup
	tempfiles := Element{}
	var file File
	if recurrent {
		err = Walk(dir.Path, &Options{
			Callback: func(osPathname string, de *Dirent) (err error) {
				wg.Add(1)
				go func() {
					if file, err = MakeFile(osPathname); err == nil {
						tempfiles.Add(file)
					}
					wg.Done()
				}()
				return nil
			},
			Unsorted:      true,
			NoHidden:      !IncHidden,
			Ignore:        IgnoreRecur,
			ScratchBuffer: make([]byte, 64*1024),
		})
	} else {
		children, err := ioutil.ReadDir(dir.Path)
		if err != nil {
			return paths, err
		}
		for _, child := range children {
			wg.Add(1)
			osPathname := path.Join(dir.Path + "/" + child.Name())
			go func() {
				if file, err = MakeFile(osPathname); err == nil {
					tempfiles.Add(file)
				}
				wg.Done()
			}()
		}
	}
	wg.Wait()
	return tempfiles.files, nil
}

func chooseFile(incFolder, incFiles, incHidden, recurrent bool, dir File) (list Files) {
	files, folder := Files{}, Files{}
	paths, err := fileList(recurrent, &dir)
	var maxPath int
	var maxSize int64
	if len(paths) == 0 || err != nil {
		return
	}
	for f := range paths {
		for i := range IgnoreSlice {
			if paths[f].Name == IgnoreSlice[i] {
				goto Exit
			}
		}
		if paths[f].IsDir() {
			if !paths[f].IsHidden() || incHidden {
				folder = append(folder, paths[f])
			}
		} else {
			if !paths[f].IsHidden() || incHidden {
				files = append(files, paths[f])
			}
		}
		if Recurrent {
			name := strings.Join(basename(ancestor(getParentPath(*paths[f])))[len(dir.Ancestors()):], "/")
			paths[f].Nick = "/" + name + "/" + paths[f].Name
		} else {
			paths[f].Nick = paths[f].Name
		}
		if len(paths[f].Nick) > maxPath {
			maxPath = len(paths[f].Nick)
		}
		if paths[f].SizeINT(DiskUse) > maxSize {
			maxSize = paths[f].SizeINT(DiskUse)
		}
	Exit:
	}
	if incFolder && !Recurrent {
		sort.Sort(folder)
		list = append(list, folder...)
	}
	if incFiles {
		sort.Sort(files)
		list = append(list, files...)
	}
	for i := range list {
		list[i].maxPath = maxPath
		list[i].maxSize = maxSize
		list[i].Number = i
	}
	return
}

func byteCountSI(b int64) string {
	const unit = 1000
	if b < unit {
		return fmt.Sprintf("%d B", b)
	}
	div, exp := int64(unit), 0
	for n := b / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB",
		float64(b)/float64(div), "kMGTPE"[exp])
}

func byteCountIEC(b int64) string {
	const unit = 1024
	if b < unit {
		return fmt.Sprintf("%d B", b)
	}
	div, exp := int64(unit), 0
	for n := b / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %ciB",
		float64(b)/float64(div), "KMGTPE"[exp])
}

func getSize(file File, dumode bool) (size int64) {
	if dumode {
		Walk(file.Path, &Options{
			Callback: func(osPathname string, de *Dirent) (err error) {
				f, err := os.Stat(osPathname)
				if err != nil {
					return
				}
				size += f.Size()
				return nil
			},
			Unsorted:      true,
			ScratchBuffer: make([]byte, 64*1024),
		})
	} else {
		size = file.File.Size()
	}
	return
}

func elements(dir string) (childs []string) {
	childs = []string{}
	if someChildren, err := ReadDirnames(dir, nil); err == nil {
		for i := range someChildren {
			childs = append(childs, dir+someChildren[i])
		}
	}
	return
}

func ancestor(dir string) (ances []string) {
	ances = append(ances, "/")
	joiner := ""
	for _, el := range strings.Split(dir, "/") {
		if el == "" {
			continue
		}
		joiner += "/" + el
		ances = append(ances, joiner)
	}
	return
}

func basename(paths []string) (names []string) {
	for i := range paths {
		names = append(names, filepath.Base(paths[i]))
	}
	return
}

func parentInfo(dir string) (parent, parentPath string) {
	parent, parentPath = "/", "/"
	if dir != "/" {
		dir = path.Clean(dir)
		parentPath, _ = path.Split(dir)
		parent = strings.TrimRight(parentPath, "/")
		_, parent = path.Split(parent)
		if parent == "" {
			parent, parentPath = "/", "/"
		}
	}
	return
}

func getParent(f File) string {
	parent, _ := parentInfo(f.Path)
	return parent
}

func getParentPath(f File) string {
	_, parentPath := parentInfo(f.Path)
	return parentPath
}

func getIcon(f File) string {
	if f.IsDir() {
		return categoryicons["folder/folder"]
	} else {
		icon := fileicons[getExte(f)]
		if icon == "" {
			return categoryicons["file/default"]
		}
		return icon
	}
}

func getMime(f File) (mime []string) {
	if f.IsDir() {
		mime = strings.Split("folder/folder", "/")
	} else {
		getmim, _, _ := DetectFile(f.Path)
		if getmim == "" {
			getmim = "file/default"
		}
		mime = strings.Split(getmim, "/")
	}
	return mime
}

func getExte(f File) string {
	if f.IsDir() {
		return "."
	} else {
		extension := path.Ext(f.Path)
		return extension
	}
}

func timespecToTime(ts syscall.Timespec) time.Time {
	return time.Unix(int64(ts.Sec), int64(ts.Nsec))
}
