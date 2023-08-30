import tkinter as tk
from tkinter import ttk
from time import sleep
from computer_vs_computerGUI import ComputerVSComputerPage

class Visual(tk.Tk):
    def __init__(self, *args, **kwargs):
        tk.Tk.__init__(self, *args, **kwargs)
        
        self.title("Chess Computer")

        # the container will all contain all frames(pages). After that we'll be able to switch between frames
        container = tk.Frame(self)
        container.pack(side="top", fill="both", expand=True)
        container.grid_rowconfigure(0, weight=1)
        container.grid_columnconfigure(0, weight=1)

        # a dictionary to add al the pages
        self.frames = {}

        frame = StartPage(container, self)
        self.frames[StartPage] = frame

        frame.grid(row=0, column=0, sticky="nsew")
        
        gameframe = ComputerVSComputerPage(container, self)
        self.frames[ComputerVSComputerPage] = gameframe
        gameframe.grid(row=0, column=0, sticky="nsew")
        
        # show the startpage
        self.show_frame(StartPage)

    def show_frame(self, cont):
        # select the correct frame
        frame = self.frames[cont]
        # raise the frame, that is, make it show on top
        frame.tkraise()


class StartPage(ttk.Frame):
    def __init__(self, parent, controller):
        ttk.Frame.__init__(self, parent)
        self.controller = controller
        
        welcome = ttk.Label(self, text="Welcome to my chess engine!", font=("Arial", 50))
        welcome.grid(row=0, column=0, pady=30, padx=20)
        
        self.make_AIvsAI_frame()
        self.AIvsAI.grid(row=1, column=0)
        
        # choose_player = ttk.Label(self, text="Choose types of player by clicking the buttons.")
        # choose_player.grid(row=1, column=0, pady=15, padx=10, columnspan=3)
        
        # choose_player1 = ttk.Label(self, text="player 1:")
        # choose_player1.grid(row=2, column=0, pady=1)
        
        # choose_player2 = ttk.Label(self, text="player 2:")
        # choose_player2.grid(row=2, column=2, pady=1)
        
        # create a button to start go to ComputerVSComputerPage
        # self.startGame = tk.Button(self, text="Start play!", command=self.to_ComputerVSComputerPage,
        #                            width=40, height=2, font=("Lucida", 12, "normal"),
        #                             bg="white")
        # self.startGame.grid(row=4, column=0, pady=80, columnspan=3)
    
    def make_AIvsAI_frame(self):
        self.AIvsAI = ttk.Frame(self)
        ttk.Label(self.AIvsAI, text="watch computers fight computers!").grid(row=0, column=0)
        ttk.Button(self.AIvsAI, text="GO!", command=lambda: self.controller.show_frame(ComputerVSComputerPage)).grid(row=0, column=1)
    
    def to_ComputerVSComputerPage(self):
        self.controller.show_frame(ComputerVSComputerPage)



if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()
