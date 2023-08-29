import tkinter as tk
from tkinter import ttk
from PIL import Image, ImageTk
from enum import Enum, auto
import numpy as np
import RustEngine as rst
from time import sleep

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
        
        gameframe = GamePage(container, self)
        self.frames[GamePage] = gameframe
        gameframe.grid(row=0, column=0, sticky="nsew")
        
        # show the startpage
        self.show_frame(StartPage)

    def show_frame(self, cont):
        # select the correct frame
        frame = self.frames[cont]
        # raise the frame, that is, make it show on top
        frame.tkraise()


class StartPage(tk.Frame):
    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent)
        self.controller = controller
        self.configure(bg="white")
                
        # keep track of the type of players we have
        self.player1Type = PlayerType.Player
        self.player2Type = PlayerType.Player
        
        welcome = tk.Label(self, width=30, height=1, font=("Helvetica", 30, "bold"), bg="white",
                          fg="black", highlightbackground="white", text="Welcome to my chess engine!")
        welcome.grid(row=0, column=0, pady=30, padx=20, columnspan=3)
        
        choose_player = tk.Label(self, width=40, height=1, font=("Helvetica", 15, "bold"), bg="white",
                          fg="black", highlightbackground="white", text="Choose types of player by clicking the buttons.")
        choose_player.grid(row=1, column=0, pady=15, padx=10, columnspan=3)
        
        choose_player1 = tk.Label(self, width=20, height=1, font=("Helvetica", 15, "normal"), bg="white",
                          fg="black", highlightbackground="white", text="player 1:")
        choose_player1.grid(row=2, column=0, pady=1)
        
        choose_player2 = tk.Label(self, width=20, height=1, font=("Helvetica", 15, "normal"), bg="white",
                          fg="black", highlightbackground="white", text="player 2:")
        choose_player2.grid(row=2, column=2, pady=1)
        
        # create a button to start playing against an engine
        self.choosePlayer1 = tk.Button(self, text="Player", command=self.swap_player1,
                                        width=15, height=2, font=("Lucida", 12, "normal"),
                                        bg="white")
        self.choosePlayer1.grid(row=3, column=0, pady=1)
        
        # create a button to start playing against another player
        self.choosePlayer2 = tk.Button(self, text="Player", command=self.swap_player2,
                                       width=15, height=2, font=("Lucida", 12, "normal"),
                                       bg="white")
        self.choosePlayer2.grid(row=3, column=2, pady=1)
        
        # create a button to start go to gamePage
        self.startGame = tk.Button(self, text="Start play!", command=self.to_GamePage,
                                   width=40, height=2, font=("Lucida", 12, "normal"),
                                    bg="white")
        self.startGame.grid(row=4, column=0, pady=80, columnspan=3)
    
    def swap_player1(self):
        match self.player1Type:
            case PlayerType.Player:
                self.choosePlayer1.configure(text="Computer")
                self.player1Type = PlayerType.Computer
            case PlayerType.Computer:
                self.choosePlayer1.configure(text="Player")
                self.player1Type = PlayerType.Player

    def swap_player2(self):
        match self.player2Type:
            case PlayerType.Player:
                self.choosePlayer2.configure(text="Computer")
                self.player2Type = PlayerType.Computer
            case PlayerType.Computer:
                self.choosePlayer2.configure(text="Player")
                self.player2Type = PlayerType.Player
        
    def to_GamePage(self):
        self.controller.show_frame(GamePage)
        self.controller.frames[GamePage].recieve_playertypes(self.player1Type, self.player2Type)



if __name__ == '__main__':
    test = Visual()
    # The main loop for the application
    test.mainloop()
