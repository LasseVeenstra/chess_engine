import tkinter as tk
from tkinter import ttk
import board_frame
from time import sleep
import threading

class Chessboard_for_AIvsAI(board_frame.ChessboardCanvas):
    def __init__(self, parent):
        super().__init__(parent)
        
        # initialize the two players as random agents
        self.chessboard_coordinator.set_player1("random")
        self.chessboard_coordinator.set_player2("random")
        
    def next_move(self):
        self.chessboard_coordinator.next_computer_move()
        self.update_board()


class ComputerVSComputerPage(ttk.Frame):
    def __init__(self, parent, controller):
        ttk.Frame.__init__(self, parent)
        self.controller = controller
        
        # main chessboard
        self.chessboard = Chessboard_for_AIvsAI(self)
        self.chessboard.grid(row=0, column=0, padx=10, pady=10)
        
        # make all the buttons
        self.make_buttons()
        self.buttons_frame.grid(row=0, column=1)
    
    # make all the buttons
    def make_buttons(self):
        self.buttons_frame = ttk.Frame(self)
        self.start_stop_button = ttk.Button(self.buttons_frame, text="start", command=self.start_stop_play)
        self.start_stop_button.grid(row=0, column=0)
        # undo button
        ttk.Button(self.buttons_frame, text="undo", command=self.chessboard.undo).grid(row=1, column=0)
        ttk.Button(self.buttons_frame, text="move", command=self.chessboard.next_move).grid(row=2, column=0)
        # reset button
        ttk.Button(self.buttons_frame, text="reset", command=self.chessboard.reset_position).grid(row=3, column=0)

    
    def playing_thread(self):
        while True:
            if self.pause_thread:
                break
            self.chessboard.next_move()
            self.update_idletasks()
            # sleep(0.01)
    
    def start_stop_play(self):
        if self.start_stop_button["text"] == "start":
            self.start_stop_button["text"] = "stop"
            self.pause_thread = False
            self.play_thread = threading.Thread(target=self.playing_thread)
            self.play_thread.start()
        else:
            self.start_stop_button["text"] = "start"
            self.pause_thread = True
            
        
        