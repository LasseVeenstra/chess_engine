import tkinter as tk
from tkinter import ttk
from time import sleep
from computer_vs_computerGUI import ComputerVSComputerPage
from analysisGUI import AnalysisPage
from play_vs_computerGUI import PlayervsAIPage

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
        
        compvcompframe = ComputerVSComputerPage(container, self)
        self.frames[ComputerVSComputerPage] = compvcompframe
        compvcompframe.grid(row=0, column=0, sticky="nsew")
        
        compvAIframe = PlayervsAIPage(container, self)
        self.frames[PlayervsAIPage] = compvAIframe
        compvAIframe.grid(row=0, column=0, sticky="nsew")
        
        analysisframe = AnalysisPage(container, self)
        self.frames[AnalysisPage] = analysisframe
        analysisframe.grid(row=0, column=0, sticky="nsew")
        
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
        
        ttk.Button(self, text="analysis board", command=lambda: self.controller.show_frame(AnalysisPage)).grid(row=2, column=0)
        
        self.make_playervsAI_frame()
        self.playervsAI.grid(row=3, column=0)
        
    def make_AIvsAI_frame(self):
        self.AIvsAI = ttk.Frame(self)
        ttk.Label(self.AIvsAI, text="watch computers fight computers!").grid(row=0, column=0)
        ttk.Button(self.AIvsAI, text="GO!", command=lambda: self.controller.show_frame(ComputerVSComputerPage)).grid(row=0, column=1)
    
    def make_playervsAI_frame(self):
        self.playervsAI = ttk.Frame(self)
        ttk.Button(self.playervsAI, text="play vs a computer!", command=lambda: self.controller.show_frame(PlayervsAIPage)).grid(row=0, column=0)
    
if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()
