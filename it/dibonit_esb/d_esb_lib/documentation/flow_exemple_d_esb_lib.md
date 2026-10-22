------- SRV_APP ----------------------------    ----------- CLI_APP --------------
app                           				 lib                          lib                   app
																				lib.connect(DBuff_connection)->Err
lib.openConnection(DBuff_connection)   
                              				 secure soket folder
							  				 create socket
							  				 on_receive().await
																				lib.publish(DBuff)->Err
								                 publish received
												 disconnect
lib.get(C1)           
	                check contract
					add tokenTo to Dbuff
					sent get (Dbuff) to soket
					