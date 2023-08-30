# import tkinter as tk
# from PIL import Image, ImageTk

# window = tk.Tk()

# def on_drag(event):
#     canvas.coords(canvas_image, event.x, event.y)

# img = ImageTk.PhotoImage(Image.open("pythonchess/images/BlackPawn.png").resize((50, 50)))
# canvas = tk.Canvas(window, width=600, height=600)
# canvas_image = canvas.create_image(300, 300, image=img, anchor="center")
# canvas.bind("<B1-Motion>", on_drag)
# canvas.pack()

# for _ in range(10):
#     canvas.create_image(300, 300, image=img)

# window.mainloop()



import tkinter as tk

GRIDCOLOR = '#7f7f7f' #same as '#%02x%02x%02x' % (127, 127, 127)

grid_left_margin = 20
grid_top_margin = 30
grid_column_width = 120
grid_row_height = 40
BORDER_WIDTH = 8
dragBox_width = grid_column_width - 20
dragBox_height = grid_row_height - 10

class DragBox(tk.Canvas):
    def __init__(self, master=None, **kwargs):
        super().__init__(master, **kwargs)
        self.make_lines()
        self._former_pos = 0,0
        self.bind('<Button>', self.remember)
        self.bind('<B1-Motion>', self.ondrag)
        self.boxes = {}
        self.active = None
        self.callback = None
        
        self.img = tk.PhotoImage(file="pythonchess/images/BlackPawn.png")

        # add the boxes
        self.add_box(80, 100, 'red', 'Box 1')
        self.add_box(80, 200, 'green', 'Box 2')
        self.add_box(200, 200, 'blue', 'Box 3')
        for i in range(10):
            self.add_box(200 + i*5, 200, 'blue', f'test {i}')

    def remember(self, event):
        self._former_pos = event.x, event.y
        self.active = None
        # for box in self.boxes:
        #     x1, y1, x2, y2 = self.coords(box)
        #     if x1 <= event.x <= x2 and y1 <= event.y <= y2:
        #         self.active = box
        self.active, = self.find_closest(event.x, event.y)
        if self.active:
            name = self.boxes.pop(self.active)
            self.boxes[self.active] = name
            self.tag_raise(self.active)

    def add_box(self, x_pos, y_pos, color, name):
        # box = self.create_rectangle(x_pos, y_pos, x_pos+dragBox_width, y_pos+dragBox_height, outline=color)
        box = self.create_image(x_pos, y_pos, image=self.img)
        self.boxes[box] = name

    def ondrag(self, event):
        if self.active is not None:
            self.move(self.active, event.x-self._former_pos[0], event.y-self._former_pos[1])
            self._former_pos = event.x, event.y
            if self.callback:
                self.callback(event.x, event.y, self.boxes[self.active])

    def make_lines(self):
        width, height = int(self['width']), int(self['height'])
        offset = BORDER_WIDTH//2
        self.create_rectangle(offset, offset, width-offset, height-offset, outline=GRIDCOLOR, width=BORDER_WIDTH)
        for row_position in range(grid_top_margin, height, grid_row_height):
            self.create_line(0, row_position, width, row_position, fill=GRIDCOLOR)
        for column_position in range(grid_left_margin, width, grid_column_width):
            self.create_line(column_position, 0, column_position, height, fill=GRIDCOLOR)

class DataFrame(tk.Frame):
    def __init__(self, master=None, **kwargs):
        super().__init__(master, **kwargs)

        self.xpos = self.make_data_display('Mouse x:')
        self.ypos = self.make_data_display('Mouse y:')
        self.boxname = self.make_data_display('Box name:')

    def make_data_display(self, label):
        column, row = self.grid_size()
        lbl = tk.Label(self, text=label)
        lbl.grid(row=row, column=0, sticky=tk.W)
        stringvar = tk.StringVar(value='[???]')
        lbl = tk.Label(self, textvariable=stringvar)
        lbl.grid(row=row, column=1, sticky=tk.W)
        return stringvar

    def update(self, xpos, ypos, name):
        self.xpos.set(xpos)
        self.ypos.set(ypos)
        self.boxname.set(name)

def main():
    root = tk.Tk()
    db = DragBox(root, bg='white', width=800, height=500)
    db.pack(side=tk.LEFT)
    data_display = DataFrame(root)
    data_display.pack(expand=True)
    db.callback = data_display.update
    root.mainloop()

if __name__ == "__main__":
    main()

# ~~~~~~~~~ the end ~~~~~~~~~